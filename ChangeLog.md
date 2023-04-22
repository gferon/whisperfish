# Changelog

v0.6.0 (2020-xx-xx)

- Using the official libsignal-client library as back-end.
- Port over to Rust, for easy linking to libsignal-client
- Add zh_CN translation, thanks to dashinfantry through !6
- Introduce `attachment_log` option
- Fix Whisperfish 0.5 outdated attachment and message handling
- Add zh_CN translation, thanks to dashinfantry through !6 and !33

[v0.5.1](https://github.com/aebruno/whisperfish/releases/tag/v0.5.1) (2018-04-01)

- Built using SailfishOSSDK-Beta-1801-Qt5-linux-64 and go v1.10
- Tested on Sailfish OS 2.1.4.14 (Lapuanjoki)
- Clear notifications when clicking on conversation (should help #76
    and #101)
- Fix #102 Enter on password dialog should accept
- Fix #100 Scrolling in select contact dialogs
- Add Finnish translation by tuokiv PR #105
- Add Hungarian translation by 1Zgp PR #110

[v0.5.0](https://github.com/aebruno/whisperfish/releases/tag/v0.5.0) (2018-03-25)

- Built using SailfishOSSDK-Beta-1801-Qt5-linux-64 and go v1.10
- Tested on Sailfish OS 2.1.4.14 (Lapuanjoki)
- Fix #26,#83 Support sending files as attachments using new content
    file pickers
- Fix #46 Add better group management. Leave group and Add group
    member are now supported (needs more testing)
- Refactor QML to only use allowed Harbour imports
- Add new contact picker for composing new messages
- Fix #86 Add setting to allow EnterKey to send messages
- Fix #92 Add option to scale image attachments
- Fix #81 Add support for opening attachments using xdg-open
- Fix #76 Close all notifications for session once one is opened
- Add Spanish translation by Caballlero PR #97

[v0.4.5](https://github.com/aebruno/whisperfish/releases/tag/v0.4.5) (2017-12-16)

- Built using SailfishOSSDK-Beta-1611-Qt5-linux-64
- Tested on Sailfish OS 2.1.3.7 (Kymijoki)
- Fix #78 add check for unsupported legacy messages

[v0.4.4](https://github.com/aebruno/whisperfish/releases/tag/v0.4.4) (2017-11-14)

- Built using SailfishOSSDK-Beta-1611-Qt5-linux-64
- Tested on Sailfish OS 2.1.2.3 (Kiiminkijoki)
- Fix #85 signal protocol updates
- Add Polish translation by paytchoo PR #84

[v0.4.3](https://github.com/aebruno/whisperfish/releases/tag/v0.4.3) (2017-04-11)

- Built using SailfishOSSDK-Beta-1611-Qt5-linux-64
- Tested on Sailfish OS 2.0.5.6 (Haapajoki)
- Fix #57, #75 Fix notifications crash from home screen
- Add support for playing video attachments
- Handle reset peer identity on incoming messages
- Tempory fix for 0 byte messages

[v0.4.2](https://github.com/aebruno/whisperfish/releases/tag/v0.4.2) (2017-03-07)

- Built using SailfishOSSDK-Beta-1611-Qt5-linux-64
- Tested on Sailfish OS 2.0.5.6 (Haapajoki)
- Fix #73 Add config option for attachment search paths
- Update German translation (PR #71)

[v0.4.1](https://github.com/aebruno/whisperfish/releases/tag/v0.4.1) (2017-03-04)

- Built using SailfishOSSDK-Beta-1611-Qt5-linux-64
- Tested on Sailfish OS 2.0.5.6 (Haapajoki)
- Fix #49 Local contacts are resolved in UI even with Shared Contacts
    disabled
- Fix #72 Add better network online detection and websocket re-connect
- Add new cover icons to show online status

[v0.4.0](https://github.com/aebruno/whisperfish/releases/tag/v0.4.0) (2017-02-14)

- Built using SailfishOSSDK-Beta-1611-Qt5-linux-64
- Tested on Sailfish OS 2.0.5.6 (Haapajoki)
- Major code refactor to use new Go QT bindings
- Viewing conversations now use native QAbstractList models which
    should improve performance
- Fix #45 The attachment directory is now configurable and can be
    changed to a location searched by the gallery
- Fix #6 and #57 Notifications no longer replace. There is a new
    notification for each message
- Fix #58 Incognito mode should be working again
- Fix #55 (partially) Add command line option for manually
    encrypting/decrypting database
- Add option to disable sharing contacts with Signal
- Fix #52 Enable quick scroll
- Add ability to resend messages
- Fix #63 Add support for numeric fingerprints
- Add CLI tool for adding extensions to attachment file names
- Add Dutch translation by d9h02f
- Add German translation by Nokius & bonanza123
- Notifications use chat instead of sms sound

[v0.3.0](https://github.com/aebruno/whisperfish/releases/tag/v0.3.0) (2016-06-07)

- Built using SailfishOSSDK-Beta-1602-Qt5-linux-64
- Tested on Sailfish OS 2.0.1.11 (Taalojärvi)
- Fix #40 Add sound/LED to notifications
- Fix #35 copy to clipboard

[v0.2.0](https://github.com/aebruno/whisperfish/releases/tag/v0.2.0) (2016-06-06)

- Second alpha release
- Built using SailfishOSSDK-Beta-1602-Qt5-linux-64
- Tested on Sailfish OS 2.0.1.11 (Taalojärvi)
- Fix #32 Keyboard closes when message arrives in active conversation
    bug
- Fix #9 Screen flickering
- Fix #25 Send button doesn\'t stay in place
- Fix #28 Remove cover action main page
- Fix #36 Fix incognito mode cancel

[v0.1.1](https://github.com/aebruno/whisperfish/releases/tag/v0.1.1) (2016-05-14)

- First alpha release
- Built using SailfishOSSDK-Beta-1511-Qt5-linux-64
- Tested on Sailfish OS 2.0.0.10 (Saimaa)
