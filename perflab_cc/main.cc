#include <iostream>
#include <vector>
#include <x86intrin.h>

#include "bitmap_image.hpp"

using ::std::array;
using ::std::vector;

struct Filter {
  int32_t divisor;
  int32_t values[9];
};

Filter gauss {
  24,
  {
    0, 4, 0,
    4, 8, 4,
    0, 4, 0
  },
};

uint64_t get_tsc() {
  uint32_t dummy;
  return __rdtscp(&dummy);
}

int main(int argc, char* argv[]) {
  bitmap_image input_bmp(argv[1]);

  // The size and pixel count of the input image
  size_t width = input_bmp.width();
  size_t height = input_bmp.height();
  size_t pixel_count = width * height;

  // Store the image data as flat arrays, one for each channel.
  vector<uint8_t> input_data(3 * pixel_count);
  for (size_t x = 1; x < input_bmp.width(); x++) {
    for (size_t y = 1; y < input_bmp.height(); y++) {
      size_t offset = y * width + x;
      input_data[0 * pixel_count + offset] = input_bmp.red_channel(x, y);
      input_data[1 * pixel_count + offset] = input_bmp.green_channel(x, y);
      input_data[2 * pixel_count + offset] = input_bmp.blue_channel(x, y);
    }
  }

  vector<uint8_t> output_data(3 * pixel_count, 0);

  uint64_t start = get_tsc();

  // This is a disgusting, unreadable loop. Never do this in real life.
  uint8_t* plane_data = input_data.data();
  uint8_t* top_row_data = plane_data + 1;
  uint8_t* current_row_data = top_row_data + width;
  uint8_t* bottom_row_data = current_row_data + width;

  for (size_t plane = 0; plane < 3; plane++) {
    for (size_t x = 1; x < width; x++) {
      uint8_t* top_row_left_data = top_row_data;
      uint8_t* current_row_left_data = current_row_data;
      uint8_t* bottom_row_left_data = bottom_row_data;
      for (size_t y = 1; y < height; y++) {

        // Shift the left for each row over one
      }
      // Wrap all the row data pointers around to the next row
      top_row_data += 5;
      current_row_data += 5;
      bottom_row_data += 5;
    }
    // Shift over a plane
    plane_data += pixel_count;
  }
  uint64_t cycles = get_tsc() - start;

  // bitmap_image output_bmp(input_bmp.width(), input_bmp.height());
}
