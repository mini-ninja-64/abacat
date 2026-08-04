#pragma once

template <typename T> void destructor(T &obj) { obj.~T(); }
template <typename T, typename... Args> T constructOnStack(Args... args) {
  return T(args...);
}
