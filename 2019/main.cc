#include <x86intrin.h>
#include <algorithm>
#include <iostream>
#include <string>
#include <unordered_map>
#include <chrono>

#include "bitmap_image.hpp"

using ::std::clamp;
using ::std::string;
using ::std::unordered_map;
using ::std::chrono::high_resolution_clock;

struct Filter {
  int32_t divisor;
  int32_t values[9];
};

// Define a few filters
unordered_map<string, Filter> filters_by_name = {
    {"gauss",
     {
         24, {0, 4, 0, 4, 8, 4, 0, 4, 0},
     }},
    {"vline",
     {
         1, {-1, 0, 1, -2, 0, 2, -1, 0, 1},
     }},
    {"hline",
     {
         1, {-1, -2, -1, 0, 0, 0, 1, 2, 1},
     }}};

// Don't do this in real life. This code only compiles on x86. It's how the
// homework assignment does micro-profiling though.
uint64_t get_tsc() {
  uint32_t dummy;
  return __rdtscp(&dummy);
}

// A sane way to handle the filtering that is unlikley to get your fired.
bitmap_image filter_a_sane_way(const bitmap_image& input_bmp,
                               const Filter& filter) {
  // The size and pixel count of the input image
  int width = input_bmp.width();
  int height = input_bmp.height();
  // The output bmp
  bitmap_image output_bmp(width, height);
  // Start counting reference cycles and apply the filter.
  uint64_t start = get_tsc();
  auto start_time = high_resolution_clock::now();
  for (int y = 1; y < height - 1; y++) {
    for (int x = 1; x < width - 1; x++) {
      // Sum the product of each 9 pixels and the filter value.
      int32_t r_total = 0, g_total = 0, b_total = 0;
      for (int j = 0; j < 3; j++) {
        for (int i = 0; i < 3; i++) {
          rgb_t pixel = input_bmp.get_pixel(x + j - 1, y + i - 1);
          int32_t filter_value = filter.values[(i * 3) + j];
          r_total += pixel.red * filter_value;
          g_total += pixel.green * filter_value;
          b_total += pixel.blue * filter_value;
        }
      }
      // Divide and clamp each to [0, 255], save it to output.
      output_bmp.set_pixel(x, y, clamp(r_total / filter.divisor, 0, 255),
                           clamp(g_total / filter.divisor, 0, 255),
                           clamp(b_total / filter.divisor, 0, 255));
    }
  }
  auto stop_time = high_resolution_clock::now();
  ::std::cout << "Ticks: " << get_tsc() - start << ::std::endl;
  ::std::cout << "Ns: " << ::std::chrono::duration_cast<::std::chrono::nanoseconds>(
                                                                                    stop_time - start_time)
  .count() << ::std::endl;
  ::std::cout << "Applying filter took: "
              << (get_tsc() - start) / (width * height) << " cycles per pixel"
              << ::std::endl;
  ::std::cout << ::std::chrono::duration_cast<::std::chrono::nanoseconds>(
                     stop_time - start_time)
                         .count() /
                     (width * height)
              << "ns\n";
  return output_bmp;
}

// The "You way over optimized this, no one can read your code and your
// coworkers all hate you" way of handling this.
bitmap_image filter_your_coworkers_hate_you_way(const bitmap_image& input_bmp,
                                                const Filter& filter) {
  // The size and pixel count of the input image
  int width = input_bmp.width();
  int height = input_bmp.height();
  int pixels = width * height;

  // Output bmp data
  uint8_t* output_data = new uint8_t[pixels * 3];

  // Pre-compute a lookup table for each filter value.
  int32_t lookup[9][256];
  for (int i = 0; i < 9; i++) {
    for (int v = 0; v < 256; v++) {
      lookup[i][v] = ((v * filter.values[i]) << 8) / filter.divisor;
    }
  }

  // Pull the data out into a flat array, by plane, row, col. Also acts to
  // pre-warm the input data.
  uint8_t* input_data = new uint8_t[pixels * 3];
  for (int y = 0; y < height; y++) {
    for (int x = 0; x < width; x++) {
      rgb_t pixel = input_bmp.get_pixel(x, y);
      input_data[(0 * pixels) + (y * width) + x] = pixel.red;
      input_data[(1 * pixels) + (y * width) + x] = pixel.green;
      input_data[(2 * pixels) + (y * width) + x] = pixel.blue;
    }
  }

  // Start counting reference cycles and apply the filter.
  uint64_t start = get_tsc();
  for (int times = 0; times < 500; times++) {
    // The offset to the upper left pixel for the 3x3 filter.
    uint8_t* ul_offset = &input_data[0];
    for (int plane = 0; plane < 3; plane++) {
      for (int y = 1; y < height - 1; y++) {
        for (int x = 1; x < width - 1; x++) {
          // Sum the product of each 9 pixels and the filter value.
          int32_t total = 0;
          uint8_t* offset = ul_offset;
          // Done without a loop
          total += lookup[0][*offset];
          total += lookup[1][*(offset + 1)];
          total += lookup[2][*(offset + 2)];
          offset += width;
          total += lookup[3][*offset];
          total += lookup[4][*(offset + 1)];
          total += lookup[5][*(offset + 2)];
          offset += width;
          total += lookup[6][*offset];
          total += lookup[7][*(offset + 1)];
          total += lookup[8][*(offset + 2)];
          // Downshift 8 (divide by 256)
          total = total >> 8;
          total = total < 0 ? -total : total;
          output_data[(pixels * plane) + (width * y) + x] = total;
          ul_offset += 1;
        }
        // Skip the wrapping 2 unused pixels
        ul_offset += 2;
      }
      // Skip the wrapping 2 rows
      ul_offset += width + width;
    }
  }
  ::std::cout << "Applying filter took: "
              << (get_tsc() - start) / (width * height) << " cycles per pixel"
              << ::std::endl;

  // Back into a bmp
  bitmap_image output_bmp(width, height);
  for (int y = 0; y < height; y++) {
    for (int x = 0; x < width; x++) {
      output_bmp.set_pixel(x, y, output_data[(0 * pixels) + (y * width) + x],
                           output_data[(1 * pixels) + (y * width) + x],
                           output_data[(2 * pixels) + (y * width) + x]);
    }
  }

  delete[] input_data;
  delete[] output_data;

  return output_bmp;
}

// Entry point.
int main(int argc, char* argv[]) {
  // Please don't actually handle args like this, use `gflags` or something.
  if (argc < 3) {
    ::std::cout << "Usage: perflab <filter> <bmp file path>" << ::std::endl;
  }
  // Load the filter from name (or panic).
  Filter filter = filters_by_name[argv[1]];
  // Load the input bmp.
  bitmap_image input_bmp(argv[2]);
  // Run the sane way to do it (and save it to disk)
  filter_a_sane_way(input_bmp, filter).save_image("sane_output.bmp");
  // filter_your_coworkers_hate_you_way(input_bmp, filter);
}
