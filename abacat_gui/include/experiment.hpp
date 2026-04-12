#pragma once

#include <cstdint>

class BasicSuper {
  public:
    explicit BasicSuper(int32_t val): xyz(val) {}

    int32_t getXyz() const {
      return xyz;
    }
  private:
    int32_t smelly = 100;
    int32_t xyz;
};

class BasicSub : public BasicSuper {
  public:
    explicit BasicSub(int32_t val): BasicSuper(val) {}
};

BasicSub create_basic_sub(int32_t xyz);

void print_basic_sub(BasicSub const &sub);

void test_func();