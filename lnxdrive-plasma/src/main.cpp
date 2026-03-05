/**
 * LNXDrive Plasma - Native KDE Plasma integration
 *
 * Provides:
 * - Dolphin file manager integration
 * - Plasmoid for system tray
 * - Qt6/QML preferences dialog
 */

#include <QApplication>
#include <QQmlApplicationEngine>
#include <KLocalizedString>
#include <KAboutData>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);

    KLocalizedString::setApplicationDomain("lnxdrive-plasma");

    KAboutData aboutData(
        QStringLiteral("lnxdrive-plasma"),
        i18n("LNXDrive"),
        QStringLiteral("0.1.0"),
        i18n("OneDrive client for KDE Plasma"),
        KAboutLicense::GPL_V3,
        i18n("(c) 2024 Strange Days Tech")
    );

    KAboutData::setApplicationData(aboutData);

    // TODO: Initialize QML engine and load main window

    return app.exec();
}
