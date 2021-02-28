/*
 * This file has been adapted from File Browser for use in Whisperfish.
 *
 * SPDX-FileCopyrightText: 2020-2021 Mirian Margiani
 * SPDX-License-Identifier: GPL-3.0-or-later OR AGPL-3.0-or-later
 */

import QtQuick 2.0
import Sailfish.Silica 1.0

Item {
    id: overlay
    z: 100
    anchors.fill: parent
    property alias title: titleLabel.text

    property bool shown: true
    opacity: shown ? 1.0 : 0.0; visible: opacity != 0.0
    Behavior on opacity { NumberAnimation { duration: 80 } }

    function show() { shown = true; }
    function hide() { shown = false; }

    Rectangle {
        anchors.top: parent.top
        height: Theme.itemSizeLarge
        width: parent.width

        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.rgba(Theme.highlightBackgroundColor, 0.5) }
            GradientStop { position: 1.0; color: "transparent" }
        }

        Label {
            id: titleLabel
            anchors.fill: parent
            anchors.margins: Theme.horizontalPageMargin
            color: Theme.highlightColor
            font.pixelSize: Theme.fontSizeLarge
            truncationMode: TruncationMode.Fade
            horizontalAlignment: Text.AlignRight
        }
    }
}
