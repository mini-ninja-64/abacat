#pragma once
#include <QColor>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QTextDocument>
#include <cstdint>

using QTextCharFormatUnderlineStyle = QTextCharFormat::UnderlineStyle;

class QSyntaxHighlighterUtil : public QSyntaxHighlighter {
  Q_OBJECT
public:
  explicit QSyntaxHighlighterUtil(QTextDocument *doc)
      : QSyntaxHighlighter(doc) {}

  void setBackground(int32_t start, int32_t count, const QColor &color) {
    QTextCharFormat format;
    format.setBackground(color);
    setFormat(start, count, format);
  }
  void setForeground(int32_t start, int32_t count, const QColor &color) {
    QTextCharFormat format;
    format.setForeground(color);
    setFormat(start, count, format);
  }
  void setUnderline(int32_t start, int32_t count, const QColor &color,
                    QTextCharFormat::UnderlineStyle style) {
    QTextCharFormat format;
    format.setUnderlineStyle(style);
    format.setUnderlineColor(color);
    mergeFormat(start, count, format);
  }

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
