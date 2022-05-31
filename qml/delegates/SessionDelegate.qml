import QtQuick 2.6
import Sailfish.Silica 1.0
import org.nemomobile.contacts 1.0
import "../components"

ListItem {
    id: delegate
    property string date: Format.formatDate(model.timestamp, _dateFormat)
    property bool isGroup: model.isGroup
    property var contact: (isGroup || !mainWindow.contactsReady) ? null : resolvePeopleModel.personByPhoneNumber(model.source, true)
    property int unreadCount: 0 // TODO implement in model
    property bool isRead: model.read // TODO investigate: is this really a bool?
    property bool isMuted: model.isMuted
    property bool isUnread: !isRead // TODO investigate: is this really a bool?
    property bool isNoteToSelf: SetupWorker.phoneNumber === model.source
    property bool isPinned: model.isPinned
    property bool isArchived: model.isArchived
    property bool hasDraft: false // TODO implement in model (#178)
    property string draft: '' // TODO implement in model (#178)
    property string profilePicture: '' // TODO implement in model (#192)
    property bool isPreviewDelivered: model.deliveryCount > 0 // TODO investigate: not updated for new message (#151, #55?)
    property bool isPreviewRead: model.readCount > 0 // TODO investigate: not updated for new message (#151, #55?)
    property bool isPreviewViewed: model.viewCount > 0 // TODO investigate: not updated for new message (#151, #55?)
    property bool isPreviewSent: model.sent // TODO cf. isPreviewReceived (#151)
    property bool hasAttachment: model.hasAttachment
    property string name: model.isGroup ? model.groupName : ( contact == null ? model.source : contact.displayLabel )
    property string message:
        (_debugMode ? "[" + model.id + "] " : "") +
        (hasAttachment && model.message === ''
            // TODO we could show an icon in front
            //: Session contains an attachment label
            //% "Attachment"
            ? qsTrId("whisperfish-session-has-attachment")
            : model.message
        )

    signal relocateItem(int sessionId)

    property bool _debugMode: SettingsBridge.boolValue("debug_mode")
    property bool _labelsHighlighted: highlighted || isUnread
    property int _dateFormat: model.section === 'older' ? Formatter.DateMedium : (model.section === 'pinned' ? Formatter.Timepoint : Formatter.TimeValue)

    contentHeight: 3*Theme.fontSizeMedium+2*Theme.paddingMedium+2*Theme.paddingSmall
    menu: contextMenuComponent
    ListView.onRemove: animateRemoval(delegate)

    function remove(contentItem) {
        //: Delete all messages from session (past tense)
        //% "All messages deleted"
        contentItem.remorseAction(qsTrId("whisperfish-session-delete-all"),
            function() {
                console.log("Deleting all messages for session: "+model.id)
                SessionModel.remove(model.index)
            })
    }

    property int clickedSessionId: 0

    // QML is faster than diesel, so well have to
    // send the item relocation signal only
    // after we get the update ourselves...
    onIsArchivedChanged: {
        if(relocationActive) {
            relocateItem(model.id)
            relocationActive = false
        }
    }

    // ...but only when it's manually activated
    // to prevent scrolled-out-of-view cases. Augh.
    property bool relocationActive: false

    function toggleReadState() {
        // TODO implement in model
        console.warn("setting read/unread is not implemented yet")
    }

    function togglePinState() {
        SessionModel.markPinned(model.index, !isPinned)
    }

    function toggleArchivedState() {
        relocationActive = true
        SessionModel.markArchived(model.index, !isArchived)
    }

    function toggleMutedState() {
        SessionModel.markMuted(model.index, !isMuted)
    }

    Item {
        anchors { fill: parent; leftMargin: Theme.horizontalPageMargin }

        ProfilePicture {
            id: profilePicContainer
            highlighted: delegate.highlighted
            labelsHighlighted: delegate._labelsHighlighted
            imageSource: profilePicture
            isNoteToSelf: delegate.isNoteToSelf
            isGroup: delegate.isGroup
            showInfoMark: isPinned || isArchived || hasDraft || isNoteToSelf || hasAttachment || isMuted
            infoMark.source: {
                if (hasDraft) 'image://theme/icon-s-edit'
                else if (isNoteToSelf) 'image://theme/icon-s-retweet' // task|secure|retweet
                else if (isPinned) 'image://theme/icon-s-high-importance'
                else if (isArchived) 'image://theme/icon-s-time'
                else if (isMuted) 'image://theme/icon-s-low-importance'
                else if (hasAttachment) 'image://theme/icon-s-attach'
                else ''
            }
            infoMark.rotation: {
                if (hasDraft) -90
                else 0
            }
            anchors {
                left: parent.left
                verticalCenter: parent.verticalCenter
            }
            onPressAndHold: delegate.openMenu()
            onClicked: {
                MessageModel.load(model.id, delegate.name)
                if (isGroup) {
                    pageStack.push(Qt.resolvedUrl("../pages/GroupProfilePage.qml"))
                } else {
                    pageStack.push(Qt.resolvedUrl("../pages/VerifyIdentity.qml"))
                }
            }
        }

        Label {
            id: upperLabel
            anchors {
                top: parent.top; topMargin: Theme.paddingMedium
                left: profilePicContainer.right; leftMargin: Theme.paddingLarge
                right: timeLabel.left; rightMargin: Theme.paddingMedium
            }
            highlighted: _labelsHighlighted
            maximumLineCount: 1
            truncationMode: TruncationMode.Fade
            text: isNoteToSelf ?
                      //: Name of the conversation with one's own number
                      //% "Note to self"
                      qsTrId("whisperfish-session-note-to-self") :
                      name
        }

        LinkedEmojiLabel {
            id: lowerLabel
            anchors {
                left: upperLabel.left; right: unreadBackground.left
                top: upperLabel.bottom; bottom: parent.bottom
            }
            wrapMode: Text.Wrap
            maximumLineCount: 2
            enableElide: Text.ElideRight
            color: highlighted ? Theme.secondaryHighlightColor :
                                 Theme.secondaryColor
            font.pixelSize: Theme.fontSizeExtraSmall
            plainText: hasDraft ?
                      //: Message preview for a saved, unsent message
                      //% "Draft: %1"
                      qsTrId("whisperfish-message-preview-draft").arg(draft) :
                      message
            highlighted: _labelsHighlighted
            verticalAlignment: Text.AlignTop
            defaultLinkActions: false
            onLinkActivated: delegate.clicked(null)
        }

        Row {
            id: timeLabel
            spacing: Theme.paddingSmall
            anchors {
                leftMargin: Theme.paddingSmall
                right: parent.right; rightMargin: Theme.horizontalPageMargin
                verticalCenter: upperLabel.verticalCenter
            }

            HighlightImage {
                source: isPreviewDelivered
                        ? "../../icons/icon-s-received.png" :
                          (isPreviewSent ? "../../icons/icon-s-sent.png" : "")
                color: Theme.primaryColor
                anchors.verticalCenter: parent.verticalCenter
                highlighted: _labelsHighlighted
                width: Theme.iconSizeSmall; height: width
            }

            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: date
                highlighted: _labelsHighlighted
                font.pixelSize: Theme.fontSizeExtraSmall
                color: highlighted ? (isUnread ? Theme.highlightColor :
                                                 Theme.secondaryHighlightColor) :
                                     (isUnread ? Theme.highlightColor :
                                                 Theme.secondaryColor)
            }
        }

        Rectangle {
            id: unreadBackground
            anchors {
                leftMargin: Theme.paddingSmall
                right: parent.right; rightMargin: Theme.horizontalPageMargin
                verticalCenter: lowerLabel.verticalCenter
            }
            visible: isUnread && unreadCount > 0
            width: isUnread ? unreadLabel.width+Theme.paddingSmall : 0
            height: width
            radius: 20
            color: profilePicContainer.profileBackgroundColor
        }

        Label {
            id: unreadLabel
            anchors.centerIn: unreadBackground
            height: 1.2*Theme.fontSizeSmall; width: height
            visible: isUnread && unreadCount > 0
            text: isUnread ? (unreadCount > 0 ? unreadCount : ' ') : ''
            font.pixelSize: Theme.fontSizeExtraSmall
            minimumPixelSize: Theme.fontSizeTiny
            fontSizeMode: Text.Fit
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            color: Theme.highlightColor
            highlighted: _labelsHighlighted
        }

        GlassItem {
            visible: isUnread
            color: Theme.highlightColor
            falloffRadius: 0.16
            radius: 0.15
            anchors {
                left: parent.left
                leftMargin: (width / -2) - Theme.horizontalPageMargin
                verticalCenter: parent.verticalCenter
            }
        }
    }

    Component {
        id: contextMenuComponent

        ContextMenu {
            id: menu

            /* MenuItem {
                text: isUnread ?
                          //: Mark conversation as 'read', even though it isn't
                          //% "Mark as read"
                          qsTrId("whisperfish-session-mark-read") :
                          //: Mark conversation as 'unread', even though it isn't
                          //% "Mark as unread"
                          qsTrId("whisperfish-session-mark-unread")
                onClicked: toggleReadState()
            } */
            MenuItem {
                text: isPinned 
                        //: 'Unpin' conversation from the top of the view
                        //% "Unpin"
                      ? qsTrId("whisperfish-session-mark-unpinned")
                        //: 'Pin' conversation to the top of the view
                        //% "Pin to top"
                      : qsTrId("whisperfish-session-mark-pinned")

                onClicked: togglePinState()
            }

            MenuItem {
                text: isMuted ?
                          //: Mark conversation as unmuted
                          //% "Unmute conversation"
                          qsTrId("whisperfish-session-mark-unmuted") :
                          //: Mark conversation as muted
                          //% "Mute conversation"
                          qsTrId("whisperfish-session-mark-muted")
                onClicked: toggleMutedState()
            }

            MenuItem {
                text: isArchived ?
                          //: Show archived messages again in the main page
                          //% "Restore to inbox"
                          qsTrId("whisperfish-session-mark-unarchived") :
                          //: Move the conversation to archived conversations
                          //% "Archive conversation"
                          qsTrId("whisperfish-session-mark-archived")
                onClicked: toggleArchivedState()
            }

            MenuItem {
                //: Delete all messages from session menu
                //% "Delete conversation"
                text: qsTrId("whisperfish-session-delete")
                onClicked: remove(delegate)
            }
        }
    }
}
