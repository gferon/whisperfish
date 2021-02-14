import QtQuick 2.2
import Sailfish.Silica 1.0

Page {
    id: group
    objectName: "group"

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height

        RemorsePopup { id: remorse }

        PullDownMenu {
            MenuItem {
                //: Add group member menu item
                //% "Add Member"
                text: qsTrId("whisperfish-group-add-member-menu")
                onClicked: {
                    remorse.execute("Changing group members unimplemented", function() {})

                    return;
                    //: Add group member remorse message
                    //% "Adding %1 to group"
                    remorse.execute(qsTrId("whisperfish-group-add-member-remorse").arg(name),
                        function() {
                            // MessageModel.addMember(SetupWorker.localId, tel)
                        }
                    )
                }
            }
            MenuItem {
                //: Leave group menu item
                //% "Leave"
                text: qsTrId("whisperfish-group-leave-menu")
                onClicked: {
                    //: Leave group remorse message
                    //% "Leaving group and removing ALL messages!"
                    remorse.execute(qsTrId("whisperfish-group-leave-remorse"),
                        function() {
                            console.log("Leaving group")
                            MessageModel.leaveGroup()
                            SessionModel.removeById(MessageModel.sessionId)
                            mainWindow.showMainPage()
                        })
                }
            }
        }

        Column {
            id: column
            width: parent.width
            spacing: Theme.paddingLarge

            PageHeader {
                title: MessageModel.peerName
            }

            SectionHeader {
                //: Group members
                //% "Group members"
                text: qsTrId("whisperfish-group-members-title")
            }

            TextArea {
                id: groupMembers
                anchors.horizontalCenter: parent.horizontalCenter
                readOnly: true
                width: parent.width
                text: {
                    // XXX code duplication with Conversation.qml
                    // Attempt to display group member names
                    var members = [];
                    var lst = MessageModel.groupMembers.split(",");
                    for(var i = 0; i < lst.length; i++) {
                        if(lst[i] != SetupWorker.localId) {
                            var member = resolvePeopleModel.personByPhoneNumber(lst[i], true);
                            if (!member) {
                                members.push(lst[i]);
                            } else {
                                members.push(member.displayLabel);
                            }
                        }
                    }
                    return members.join(", ");
                }
            }
        }
    }
}
