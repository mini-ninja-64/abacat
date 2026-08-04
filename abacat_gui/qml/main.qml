import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import Qt.labs.qmlmodels

import zone.minis.abacat 1.0

ApplicationWindow {
    id: root
    height: 480
    title: qsTr("abacat")
    visible: true
    width: 640

    FontMetrics {
        id: fontMetrics
        font.family: "Monaco"
    }

    AbacatDocument {
        id: abacatDocument
    }
    Rectangle {
        anchors.fill: parent
        color: abacatDocument.backgroundColor()

        ListView {
            id: listView
            property real maxTextWidth: 0
            property int longestLine: 0
            property int longestLength: 0
            property real minimumTextEditWidth: 50

            boundsBehavior: Flickable.StopAtBounds
            boundsMovement: Flickable.StopAtBounds

            anchors.fill: parent
            Layout.fillWidth: true

            spacing: 0
            focus: true
            model: abacatDocument
            KeyNavigation.priority: KeyNavigation.BeforeItem

            delegate: Row {
                id: row
                required property int index
                required property string expression
                required property var answer
                required property var model
                required property bool renderAsError
                required property var errorRange

                spacing: 10

                Text {
                    width: 50
                    color: abacatDocument.plainTextColor()
                    font.family: "Monaco"

                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignRight
                    anchors.verticalCenter: parent.verticalCenter

                    text: index
                }

                QmlAbacatSyntaxHighlighter2 {
                    document: textEditLine.textDocument
                    error_range: errorRange
                }

                TextEdit {
                    id: textEditLine
                    property bool isInitialized: false
                    Component.onCompleted: isInitialized = true

                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignLeft
                    anchors.verticalCenter: parent.verticalCenter

                    color: abacatDocument.plainTextColor()
                    selectedTextColor: palette.highlightedText
                    font.family: "Monaco"
                    text: expression
                    width: listView.maxTextWidth < listView.minimumTextEditWidth ? listView.minimumTextEditWidth : listView.maxTextWidth

                    onFocusChanged: focused => {
                        if (focused) {
                            abacatDocument.setLine(index);
                            // move cursor to end??
                        }
                    }

                    function moveUp() {
                        if (abacatDocument.currentLine > 0) {
                            abacatDocument.setLine(abacatDocument.currentLine - 1);
                        }
                    }
                    function moveDown() {
                        abacatDocument.setLine(abacatDocument.currentLine + 1);
                    }
                    Keys.onBacktabPressed: event => {
                        moveUp();
                        event.accepted = true;
                    }
                    Keys.onTabPressed: event => {
                        moveDown();
                        event.accepted = true;
                    }
                    Keys.onReturnPressed: event => {
                        event.accepted = true;
                        abacatDocument.insertRow(index + 1, "");
                        moveDown();
                    }
                    Keys.onUpPressed: moveUp()
                    Keys.onDownPressed: moveDown()

                    KeyNavigation.priority: KeyNavigation.BeforeItem

                    Keys.onPressed: event => {
                        if ((event.key === Qt.Key_Backspace) && (event.modifiers & Qt.ControlModifier)) {
                            textEditLine.text = textEditLine.text.slice(cursorPosition);
                            cursorPosition = 0;
                            event.accepted = true;
                        }
                    }
                    focus: abacatDocument.currentLine === index
                    activeFocusOnPress: true
                    activeFocusOnTab: true
                    focusPolicy: Qt.TabFocus
                    onTextChanged: {
                        if (!isInitialized) {
                            return;
                        }
                        const content = getText(0, length);
                        abacatDocument.setExpr(index, content, abacatDocument.index(index, 0));

                        if (content.length > listView.longestLength || listView.longestLine === index) {
                            let maxStrWidth = 0;
                            for (let i = 0; i < listView.count; i++) {
                                const lineExpr = listView.model.getExpr(i);
                                const fontWidth = fontMetrics.advanceWidth(lineExpr);

                                if (fontWidth > maxStrWidth) {
                                    listView.longestLine = i;
                                    listView.longestLength = lineExpr.length;
                                    maxStrWidth = fontWidth;
                                }
                            }
                            listView.maxTextWidth = maxStrWidth;
                        }
                    }
                }

                // TODO: MOVE TO DUNAMIC COMPONENT WIV LOADER
                Text {
                    opacity: abacatDocument.answerOpacity()
                    font.family: "Monaco"

                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignLeft
                    anchors.verticalCenter: parent.verticalCenter

                    color: abacatDocument.plainTextColor()
                    // anchors.verticalCenter: parent.verticalCenter

                    text: answer === undefined ? "" : "="
                    // height: 50
                }
                QmlAbacatSyntaxHighlighter2 {
                    document: textEditAnswer.textDocument
                    render_as_error: renderAsError
                }
                TextEdit {
                    id: textEditAnswer
                    property bool processing: false
                    opacity: abacatDocument.answerOpacity()
                    font.family: "Monaco"

                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignLeft
                    anchors.verticalCenter: parent.verticalCenter

                    color: abacatDocument.plainTextColor()
                    // anchors.verticalCenter: parent.verticalCenter

                    text: answer === undefined ? "" : answer
                    readOnly: true
                    selectByMouse: true
                }
            }
        }
    }
}
