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

    MyObject {
        id: myObject
    }
    Rectangle {
        anchors.fill: parent
        color: myObject.backgroundColor()

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
            clip: true
            model: myObject
            KeyNavigation.priority: KeyNavigation.BeforeItem

            delegate: Row {
                id: row
                required property int index
                required property string expression
                required property var answer
                // property var nextSection
                // property var previousSection

                required property var model

                spacing: 10

                Text {
                    leftPadding: 2
                    rightPadding: 2
                    width: 50
                    color: palette.text
                    font.family: "Monaco"
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignRight
                    text: index
                    anchors.verticalCenter: parent.verticalCenter

                    // anchors.horizontalCenter: parent.horizontalCenter
                    // Layout.fillHeight: true
                }

                TextEdit {
                    property bool processing: false
                    // width: 10
                    // Layout.fillWidth: true

                    color: palette.text
                    selectedTextColor: palette.highlightedText
                    font.family: "Monaco"
                    textFormat: TextEdit.RichText
                    text: expression
                    width: listView.maxTextWidth < listView.minimumTextEditWidth ? listView.minimumTextEditWidth : listView.maxTextWidth

                    onFocusChanged: focused => {
                        if (focused) {
                            myObject.setLine(index);
                            // move cursor to end
                        }
                    }

                    function moveUp() {
                        if (myObject.currentLine > 0) {
                            myObject.setLine(myObject.currentLine - 1);
                        }
                    }
                    function moveDown() {
                        myObject.setLine(myObject.currentLine + 1);
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
                        myObject.insertRow(index + 1, "", myObject.index(index + 1, 0));
                        moveDown();
                    }
                    Keys.onUpPressed: moveUp()
                    Keys.onDownPressed: moveDown()

                    KeyNavigation.priority: KeyNavigation.BeforeItem

                    focus: myObject.currentLine === index
                    activeFocusOnPress: true
                    activeFocusOnTab: true
                    focusPolicy: Qt.TabFocus
                    onTextChanged: {
                        // console.log(Object.keys(parent.parent));
                        const content = getText(0, length);
                        myObject.setExpr(index, content, myObject.index(index, 0));

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

                        // TODO: blerguhhhh dont like this, maybe move to rust layer??? is QSyntaxHighlighter usable?
                        if (!processing) {
                            processing = true;
                            let p = cursorPosition;
                            text = myObject.syntaxHighlight(content);
                            cursorPosition = p;
                            processing = false;
                        }
                    }
                }

                // TODO: MOVE TO DUNAMIC COMPONENT WIV LOADER
                Text {
                    color: palette.text
                    font.family: "Monaco"
                    leftPadding: 2
                    rightPadding: 2
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignRight
                    // anchors.verticalCenter: parent.verticalCenter

                    text: answer === undefined ? "" : "="
                }
                TextEdit {
                    property bool processing: false

                    color: palette.text
                    font.family: "Monaco"
                    leftPadding: 2
                    rightPadding: 2
                    verticalAlignment: Text.AlignVCenter
                    horizontalAlignment: Text.AlignRight
                    // anchors.verticalCenter: parent.verticalCenter

                    text: answer === undefined ? "" : answer
                    readOnly: true
                    selectByMouse: true
                    textFormat: TextEdit.RichText
                    // TODO: Syntax Highlighting
                }
            }
        }
    }
}
