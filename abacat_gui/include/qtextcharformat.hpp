#pragma once

#include "rust/cxx.h"
#include <QBrush>
#include <QTextCharFormat>
#include <QTextFormat>

using QTextCharFormatUnderlineStyle = QTextCharFormat::UnderlineStyle;
using QTextFormatFormatType = QTextFormat::FormatType;

class QTextCharFormatPatched : public QTextCharFormat {
public:
  explicit QTextCharFormatPatched() : QTextCharFormat() {}

  // Note: Done to save me having to implement QBrush in Rust for now
  void setForegroundColor(const QColor &color) { setForeground(color); }
  QColor foregroundColor() const {
    auto f = foreground();
    return f.color();
  }
  void setBackgroundColor(const QColor &color) { setBackground(color); }
  QColor backgroundColor() const {
    auto b = background();
    return b.color();
  }
};

template <>
struct rust::IsRelocatable<QTextCharFormatPatched> : std::true_type {};
