#include <x86intrin.h>
#include <algorithm>
#include <iostream>
#include <string>
#include <unordered_map>

#include "bitmap_image.hpp"

using ::std::clamp;
using ::std::string;
using ::std::unordered_map;
using ::std::vector;

struct Filter {
  int32_t divisor;
  int32_t values[9];
};

// Define a few filters
unordered_map<string, Filter> filters_by_name = {
    {"gauss",
     {
         24,
         {0, 4, 0, 4, 8, 4, 0, 4, 0},
     }},
    {"vline",
     {
         1,
         {-1, 0, 1, -2, 0, 2, -1, 0, 1},
     }},
    {"hline",
     {
         1,
         {-1, -2, -1, 0, 0, 0, 1, 2, 1},
     }}};

// Don't do this in real life. This code only compiles on x86. It's how the
// homework assignment does micro-profiling though.
uint64_t get_tsc() {
  uint32_t dummy;
  return __rdtscp(&dummy);
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
  // The size and pixel count of the input image
  int width = input_bmp.width();
  int height = input_bmp.height();
  // The output bmp
  bitmap_image ouput_bmp(width, height);
  // Start counting reference cycles and apply the filter.
  uint64_t start = get_tsc();
  for (int y = 1; y < height; y++) {
    for (int x = 1; x < width; x++) {
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
      ouput_bmp.set_pixel(x, y, clamp(r_total / filter.divisor, 0, 255),
                          clamp(g_total / filter.divisor, 0, 255),
                          clamp(b_total / filter.divisor, 0, 255));
    }
  }
  ::std::cout << "Applying filter took: "
              << (get_tsc() - start) / (width * height) << " cycles per pixel"
              << ::std::endl;
  // Save output bmp to disk
  ouput_bmp.save_image("output.bmp");
}
