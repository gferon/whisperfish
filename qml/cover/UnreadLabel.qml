import QtQuick 2.2
import Sailfish.Silica 1.0

Item {
    width: parent.width
    height: unreadLabel.height + unreadLabel.y
    Label {
        id: titleLabel
        text: "Whisperfish"
        width: parent.width
        color: Theme.highlightColor
        font.pixelSize: Theme.fontSizeSmall
        truncationMode: TruncationMode.Fade
        anchors {
            left: parent.left
            right: unreadLabel.left
            bottom: statusLabel.top
            bottomMargin: -Theme.paddingSmall
        }
    }
    Label {
        id: statusLabel
        //: Cover new message label
        //% "New"
        text: qsTrId("whisperfish-cover-new-label")
        opacity: 0.6
        width: parent.width
        visible: SessionModel.unread > 0
        font.pixelSize: Theme.fontSizeExtraSmall
        truncationMode: TruncationMode.Fade
        color: Theme.highlightColor
        anchors {
            left: parent.left
            right: unreadLabel.left
            baseline: unreadLabel.baseline
        }
    }
    Label {
        id: unreadLabel
        y: Theme.paddingMedium
        color: Theme.primaryColor
        text: SessionModel.unread ? SessionModel.unread : ''
        font {
            pixelSize: Theme.fontSizeHuge
            family: Theme.fontFamilyHeading
        }
        anchors.right: parent.right
    }
}
