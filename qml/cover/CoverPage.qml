import QtQuick 2.2
import Sailfish.Silica 1.0

CoverBackground {
    CoverActionList {
        id: coverAction
        CoverAction {
            iconSource: "image://theme/icon-cover-message"
            onTriggered: {
                if(!SetupWorker.locked) {
                    mainWindow.activate()
                    mainWindow.newMessage(PageStackAction.Immediate)
                }
            }
        }
    }

    Label {
        id: unreadCount
        text: SessionModel.unread
        x: Theme.paddingLarge
        y: Theme.paddingMedium
        font.pixelSize: Theme.fontSizeHuge

        visible: SessionModel.unread > 0
    }

    Label {
        id: unreadLabel

        //: Unread messages count cover label. Code requires exact line break tag "<br/>".
        //% "Unread<br/>message(s)"
        text: qsTrId("whisperfish-cover-unread-label", SessionModel.unread).replace("<br/>", "\n")
        font.pixelSize: Theme.fontSizeExtraSmall
        maximumLineCount: 2
        wrapMode: Text.Wrap
        fontSizeMode: Text.HorizontalFit
        lineHeight: 0.8
        height: implicitHeight/0.8
        verticalAlignment: Text.AlignVCenter

        visible: SessionModel.unread > 0
        anchors {
            right: parent.right
            left: unreadCount.right
            leftMargin: Theme.paddingMedium
            baseline: unreadCount.baseline
            baselineOffset: lineCount > 1 ? -implicitHeight/2 : -(height-implicitHeight)/2
        }
    }


    OpacityRampEffect {
        offset: 0.5
        sourceItem: unreadLabel
        enabled: unreadLabel.implicitWidth > Math.ceil(unreadLabel.width)
    }

    Column {
        id: contentColumn

        x: Theme.paddingLarge
        spacing: Theme.paddingSmall
        width: parent.width - 2*Theme.paddingLarge

        anchors {
            top: unreadLabel.visible ? unreadLabel.bottom : unreadLabel.top
            bottom: coverActionArea.top
        }

        Item {
            width: parent.width + Theme.paddingLarge
            height: parent.height - parent.spacing

            ListView {
                id: sessionList

                // XXX Maybe we can use a delegate model to resort without pinning.
                //     Pins don't make a lot of sense in this context
                model: SessionModel

                width: parent.width
                height: parent.height - Theme.paddingSmall
                spacing: Theme.paddingSmall

                delegate: SessionItem {
                    session: model
                    width: sessionList.width
                }
            }
        }
    }

    OpacityRampEffect {
        offset: 0.7
        sourceItem: contentColumn
        direction: OpacityRamp.TopToBottom
    }

    Image {
        source: {
            if(SessionModel.unread > 0) {
                return "/usr/share/harbour-whisperfish/icons/172x172/gold.png"
            } else if (ClientWorker.connected) {
                return "/usr/share/harbour-whisperfish/icons/172x172/connected.png"
            } else if (!ClientWorker.connected) {
                return "/usr/share/harbour-whisperfish/icons/172x172/disconnected.png"
            } else {
                return "/usr/share/icons/hicolor/172x172/apps/harbour-whisperfish.png"
            }
        }
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width
        height: sourceSize.height * (parent.width / sourceSize.width)
        fillMode: Image.PreserveAspectFit
        opacity: 0.1
    }
}
