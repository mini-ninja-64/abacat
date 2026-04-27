#pragma once

#include "rust/cxx.h"
#include <QBrush>
#include <QColor>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QTextCursor>
#include <QTextDocument>
#include <QTextFormat>

class QSyntaxHighlighterPatched : public QSyntaxHighlighter {
  Q_OBJECT
public:
  explicit QSyntaxHighlighterPatched(QObject *obj = nullptr)
      : QSyntaxHighlighter(obj) {}

  void highlightBlock(const QString &) override {}

  void mergeFormat(int start, int count, const QTextCharFormat &formatToApply) {
    if (start < 0)
      return;

    const int end = start + count;
    for (int i = start; i < end; ++i) {
      auto format = this->format(i);
      format.merge(formatToApply);
      this->setFormat(i, 1, format);
    }
  }
};

// template <>
// struct rust::IsRelocatable<QSyntaxHighlighterPatched> : std::true_type {};
