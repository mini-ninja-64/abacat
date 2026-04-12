#include "experiment.hpp"

#include <iostream>
#include <utility>

BasicSub create_basic_sub(int32_t xyz) {
    BasicSub b(xyz);
    return b;
}

void print_basic_sub(BasicSub const &sub) {
    std::cout << sub.getXyz() << std::endl;
}

void test_func() {

}