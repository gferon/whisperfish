/*
 * Copyright (C) 2012-2015 Jolla Ltd.
 *
 * The code in this file is distributed under multiple licenses, and as such,
 * may be used under any one of the following licenses:
 *
 *   - GNU General Public License as published by the Free Software Foundation;
 *     either version 2 of the License (see LICENSE.GPLv2 in the root directory
 *     for full terms), or (at your option) any later version.
 *   - GNU Lesser General Public License as published by the Free Software
 *     Foundation; either version 2.1 of the License (see LICENSE.LGPLv21 in the
 *     root directory for full terms), or (at your option) any later version.
 *   - Alternatively, if you have a commercial license agreement with Jolla Ltd,
 *     you may use the code under the terms of that license instead.
 *
 * You can visit <https://sailfishos.org/legal/> for more information
 */

/*
 * Modifications for Whisperfish:
 * SPDX-FileCopyrightText: 2021 Mirian Margiani
 * SPDX-License-Identifier: AGPL-3.0-or-later
 *
 */

import QtQuick 2.6
import Sailfish.Silica 1.0
import "../delegates"

SilicaListView {
    id: messagesView

    // TODO verify: date->string is always ISO formatted?
    // TODO Use a custom property for sections. It should contain
    // at least 1) date, 2) unread boundary, 3) ...
    // This would allow us to show sticky sections.
    //     section.labelPositioning: ViewSection.InlineLabels
    //     -> change to ViewSection.CurrentLabelAtStart if
    //        messageList.moving for a certain time
    section.property: "timestamp"

    property bool menuOpen: false
    property var selectedMessages: ({}) // changed by assignment in resetSelection()/onItemSel...Toggled
    property int selectedCount: 0
    property bool isSelecting: false
    property bool selectionBlocked: false
    property bool hideSelected: false
    property var __running_remorse: null

    signal replyTriggered(var index, var modelData)
    signal quoteClicked(var clickedIndex, var quotedData)
    signal itemSelectionToggled(var modelData)

    function startSelection() {
        isSelecting = true
    }

    function resetSelection(keepRemorse) {
        if (!keepRemorse && !!__running_remorse) {
            __running_remorse.triggered.disconnect(resetSelection)
            __running_remorse.canceled.disconnect(resetSelection)
            __running_remorse.cancel()
        }
        isSelecting = false
        selectedMessages = {}
        selectedCount = 0
        selectionBlocked = false
        hideSelected = false
        if (!keepRemorse) __running_remorse = null
    }

    // Use this function to call a message action. It makes sure to
    // block the selection while the action is executed, and resets
    // the selection afterwards.
    function messageAction(callback, listItem) {
        selectionBlocked = true
        var remorse = callback(listItem)

        if (!!remorse) { // delay resetting if the action returns a remorse item
            __running_remorse = remorse
            remorse.triggered.connect(function(){resetSelection(true)})
            remorse.canceled.connect(resetSelection)
        } else {
            resetSelection()
        }
    }

    // ↓↓↓↓ message action functions to be used through messageAction(action, ...)
    //      Action functions may take a list item as argument to e.g. execute a
    //      remorse action on it.
    //      Actions may return a remorse item object. If so, messageAction() will
    //      reset the selection only after the remorse item has finished, however
    //      *before* the remorse item's callback has been executed. I.e. the
    //      selection cannot be used within the remorse callback.

    function reactInline(listItem) { // call through messageAction()
        listItem.remorseAction("Emoji reactions are not yet implemented.", function(){})
    }

    function resendInline(listItem) { // call through messageAction()
        // TODO give some kind of feedback on success
        console.log("Resending message:", listItem.modelData.id)
        MessageModel.sendMessage(listItem.modelData.id) // no remorse needed
    }

    function resendSelected() { // call through messageAction()
        // TODO give some kind of feedback on success
        Remorse.popupAction(root, "Resending selected messages is not yet implemented.", function(){})
    }

    function copyInline(listItem) { // call through messageAction()
        // TODO give some kind of feedback on success
        Clipboard.text = listItem.modelData.message
    }

    function copySelected() { // call through messageAction()
        // TODO give some kind of feedback on success
        // TODO implement getting a message by its ID from the model

        // Copying multiple messages should copy them sorted by message id
        // and including sender name and timestamp:
        // [2021-02-25T00:00:00] Jane Doe: hello John
        // [2021-02-25T00:00:01] John Doe: hello Jane
        Remorse.popupAction(root, "Copying selected messages is not yet implemented.", function(){})
    }

    function forwardInline(listItem) { // call through messageAction()
        // TODO implement: a list of contacts should be openend where
        // the user can select one or multiple recipients (this can probably
        // use the same implementation needed for sharing, #242)
        listItem.remorseAction("Forwarding messages is not yet implemented.", function(){})
    }

    function deleteSelectedForSelf() { // call through messageAction()
        var selectedIndices = _getSelectedIndices()
        hideSelected = true

        return Remorse.popupAction(
            //: Remorse: *locally* deleted one or multiple message (past tense)
            //% "Locally deleted %n message(s)"
            root, qsTrId("whisperfish-remorse-deleted-messages-locally", selectedCount),
            function() {
                for (var i in selectedIndices) {
                    console.log("Delete message:", selectedIndices[i])
                    // TODO MessageModel.remove should take a message ID.
                    // Rewrite this function to use IDs when that is fixed.
                    MessageModel.remove(selectedIndices[i])
                }
            })
    }

    function deleteSelectedForAll() { // call through messageAction()
        // TODO implement in the model
        Remorse.popupAction(root, "Deleting for all peers is not yet implemented.", function(){})
    }

    function showMessageInfo() { // call through messageAction()
        // TODO implement: open a separate page and show some info on
        // the currently selected message. This requires direct access to
        // message data through the model. It should silently fail
        // when multiple messages are selected.
        Remorse.popupAction(root, "Message info is not yet implemented.", function(){})
    }

    /* ↑↑↑↑ message action functions to be used through messageAction(action, ...) */

    function _getSelectedIndices() {
        var selectedIndices = []
        for (var i in selectedMessages) {
            if (!selectedMessages.hasOwnProperty(i)) continue
            selectedIndices.push(selectedMessages[i].index)
        }
        selectedIndices.sort(function(a,b){return b-a}) // descending
        return selectedIndices
    }

    // TODO all model methods must take message id's instead of
    // indices. When that is given, we can remove this line and
    // keep the selection.
    // WARNING It is problematic to reset the selection while
    // a message action is running. The selection should only be reset through
    // messageAction(), and all actions should use IDs instead of indices.
    onCountChanged: resetSelection()
    onSelectedCountChanged: if (selectedCount === 0) isSelecting = false
    onItemSelectionToggled: {
        if (selectedMessages[modelData.id] === undefined) {
            selectedMessages[modelData.id] = {id: modelData.id, index: modelData.index}
            selectedCount++
        } else {
            delete selectedMessages[modelData.id]
            selectedCount--
        }
        selectedMessages = selectedMessages // notify changes
    }

    verticalLayoutDirection: ListView.BottomToTop
    quickScroll: true  // TODO how to only allow downwards?
    currentIndex: -1
    highlightFollowsCurrentItem: false
    delegate: Item {
        id: wrapper
        property string newerSection: ListView.previousSection
        property string olderSection: ListView.nextSection
        property bool atSectionBoundary: {
            // Section strings are ISO formatted timestamps.
            // E.g. '2021-02-07T22:00:01'
            ((olderSection.substr(0, 10) !== "" &&
             olderSection.substr(0, 10) !== ListView.section.substr(0, 10))
                    || model.index === ListView.view.count - 1) ?
                        true : false
        }
        property Item section // overrides the default section item
        property bool isServiceMessage: false // TODO implement in backend

        height: loader.y + loader.height
        width: parent.width

        ListView.onIsCurrentItemChanged: {
            if (!ListView.isCurrentItem) return
            blinkAnimation.start()
        }
        ListView.onRemove: loader.item.animateRemoval(wrapper)
        onAtSectionBoundaryChanged: {
            if (atSectionBoundary) {
                section = sectionHeaderComponent.createObject(wrapper, {
                    'title': ListView.section.substr(0, 10) === '' ?
                                 (newerSection.substr(0, 10) === '' ?
                                      MessageModel.peerName : newerSection) :
                                 Format.formatDate(ListView.section, Formatter.DateFull)
                })
            } else {
                // We manually remove the section item if it does not
                // match our criteria. ListView.sections.criteria is not
                // versatile enough. Using a Loader as section.delegate is
                // not possible because we wouldn't have access to section data.
                section.destroy()
                section = null
            }
        }

        SequentialAnimation {
            id: blinkAnimation
            FadeAnimator { target: wrapper; duration: 220; from: 1.0; to: Theme.opacityHigh }
            FadeAnimator { target: wrapper; duration: 200; from: Theme.opacityHigh; to: 1.0 }
            FadeAnimator { target: wrapper; duration: 180; from: 1.0; to: Theme.opacityHigh }
            FadeAnimator { target: wrapper; duration: 180; from: Theme.opacityHigh; to: 1.0 }
        }

        Loader {
            id: loader
            y: section ? section.y + section.height : 0
            width: parent.width

            // choose the delegate based on message contents
            // NOTE we could make this loader asynchronous if we find a way
            // to calculate the effective message height here
            sourceComponent: wrapper.isServiceMessage ?
                                 serviceMessageDelegate :
                                 defaultMessageDelegate
        }

        Component {
            id: defaultMessageDelegate
            MessageDelegate {
                // necessary to make implicit properties available
                modelData: model
                menu: messageContextMenu
                // set explicitly because attached properties are not available
                // inside the loaded component
                listView: messagesView
                replySignal: replyTriggered
                quoteClickedSignal: quoteClicked
                openMenuOnPressAndHold: isSelecting ? false : true

                onClicked: {
                    if (isSelecting && !selectionBlocked) {
                        itemSelectionToggled(model)
                    }
                }
            }
        }

        Component {
            id: serviceMessageDelegate
            ServiceMessageDelegate {
                // necessary to make implicit properties available
                modelData: model
            }
        }
    }

    Component {
        id: sectionHeaderComponent
        Column {
            id: sectionHeader
            property string title: ""
            spacing: 0
            width: parent.width

            Item { width: 1; height: Theme.paddingLarge }

            Label {
                width: parent.width - 4*Theme.horizontalPageMargin
                anchors.horizontalCenter: parent.horizontalCenter
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                color: Theme.highlightColor
                text: parent.title

                Separator {
                    anchors {
                        horizontalCenter: parent.horizontalCenter
                        top: parent.baseline; topMargin: 8
                    }
                    width: parent.width-2*Theme.horizontalPageMargin
                    horizontalAlignment: Qt.AlignHCenter
                    color: Theme.highlightColor
                }
            }

            Item { width: 1; height: Theme.paddingLarge }
        }
    }

    VerticalScrollDecorator { flickable: messagesView }

    function jumpToMessage(index) {
        // TODO This should use the message id instead of an index.
        //      Indices may change, the saved index may become invalid.
        //      We need a method like MessageModel.indexFromId(mId) to
        //      get the current and valid index for the quoted message.
        positionViewAtIndex(index, ListView.End)

        // We briefly set the current index to the target message. This
        // notifies the resp. delegate which is enough to start an animation.
        // We reset it to -1 because of this comment in the old implementation:
        // "avoids resetting focus every time a row is added, which breaks text input"
        currentIndex = index
        currentIndex = -1
    }

    Component {
        id: messageContextMenu

        // IMPORTANT:
        // The context menu should show at max. 4 entries at a time.
        // Dangerous, destructiv, or secondary actions should be shown in
        // the selection/actions panel (defined in ConversationPage.qml).
        // See ConversationPage::actionsPanel for a list of message actions.

        ContextMenu {
            id: menu
            width: parent ? parent.width : Screen.width

            onActiveChanged: {
                // We can safely assume only one context menu is open at a time.
                // We assume less safely that this is the only menu component that will
                // ever be available in a message view.
                if (active) messagesView.menuOpen = true
                else messagesView.menuOpen = false
            }

            MenuItem {
                //: React with emoji to message menu item
                //% "React"
                text: qsTrId("whisperfish-react-message-menu")
                visible: !!(menu.parent && !menu.parent.modelData.queued) // TODO use .failed
                onClicked: reactInline(menu.parent)
            }
            MenuItem {
                //: Resend message menu item
                //% "Retry sending"
                text: qsTrId("whisperfish-resend-message-menu")
                visible: !!(menu.parent && menu.parent.modelData.queued)
                onClicked: resendInline(menu.parent)
            }
            MenuItem {
                //: Copy message menu item
                //% "Copy"
                text: qsTrId("whisperfish-copy-message-menu")
                visible: menu.parent && menu.parent.hasText
                onClicked: copyInline(menu.parent)
            }
            MenuItem {
                //: Forward message menu item
                //% "Forward"
                text: qsTrId("whisperfish-forward-message-menu")
                visible: !!(menu.parent && !menu.parent.modelData.queued) // TODO use .failed
                onClicked: forwardInline(menu.parent)
            }
            MenuItem {
                //: "Select and show more options" message menu item
                //% "Select • more"
                text: qsTrId("whisperfish-select-or-options-message-menu")
                onClicked: {
                    itemSelectionToggled(menu.parent.modelData)
                    messagesView.startSelection()
                }
            }
        }
    }
}
