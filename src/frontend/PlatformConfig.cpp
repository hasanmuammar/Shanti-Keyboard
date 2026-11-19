/*
 *  Shanti Keyboard
 *  Copyright (C) 2025 Muhammad Mominul Huque <mominul2082@gmail.com>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 */

#include <QProcess>
#include <QProcessEnvironment>
#include <QStringList>

#include "PlatformConfig.h"
#include "Log.h"

DesktopEnvironment detectDesktopEnvironment() {
    const auto env = QProcessEnvironment::systemEnvironment();
    const QString desktops = QStringList{
        env.value("XDG_CURRENT_DESKTOP"),
        env.value("DESKTOP_SESSION"),
        env.value("GDMSESSION"),
        env.value("XDG_SESSION_DESKTOP")
    }.join(';').toLower();

    return desktops.contains("gnome") ? DesktopEnvironment::GNOME
                                       : DesktopEnvironment::Unknown;
}

QString desktopEnvironmentToString(DesktopEnvironment de) {
    return de == DesktopEnvironment::GNOME ? QStringLiteral("GNOME")
                                           : QStringLiteral("Unknown");
}

bool isAppIndicatorEnabled() {
    QProcess process;
    process.start("gnome-extensions", {"list", "--enabled"});
    if (!process.waitForFinished() || process.exitStatus() != QProcess::NormalExit ||
        process.exitCode() != 0) {
        return false;
    }

    const QStringList extensions = QString::fromUtf8(process.readAllStandardOutput())
                                       .split('\n', Qt::SkipEmptyParts);
    return extensions.contains("appindicatorsupport@rgcjonas.gmail.com") ||
           extensions.contains("ubuntu-appindicators@ubuntu.com");
}

bool shouldShowTrayIcon() {
    return detectDesktopEnvironment() != DesktopEnvironment::GNOME || isAppIndicatorEnabled();
}

namespace {
QString readDConfKey(const QString &key) {
    QProcess process;
    process.start("dconf", {"read", key});
    process.waitForFinished();
    if (process.exitStatus() == QProcess::NormalExit && process.exitCode() == 0)
        return QString::fromUtf8(process.readAllStandardOutput()).trimmed();
    LOG_ERROR("Could not read GNOME input sources from dconf\n");
    return {};
}

void writeDConfSetting(const QString &key, const QString &value) {
    QProcess process;
    process.start("dconf", {"write", key, value});
    process.waitForFinished();
    if (process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0)
        LOG_ERROR("Could not update GNOME input sources in dconf\n");
}

QStringList parseInputSources(QString raw) {
    raw.remove('[');
    raw.remove(']');
    QStringList sources;
    while (true) {
        const auto index = raw.indexOf("),");
        if (index < 0)
            break;
        sources.append(raw.left(index + 1).trimmed());
        raw.remove(0, index + 2);
    }
    if (!raw.trimmed().isEmpty())
        sources.append(raw.trimmed());
    return sources;
}

void setupGnomeInputSource() {
    const QString key = QStringLiteral("/org/gnome/desktop/input-sources/sources");
    auto sources = parseInputSources(readDConfKey(key));
    if (sources.contains("('ibus', 'Shanti')"))
        return;

    const QString firstSource = sources.isEmpty() ? QStringLiteral("('xkb', 'us')") : sources.first();
    sources.append("('ibus', 'Shanti')");
    writeDConfSetting(key, QStringLiteral("[%1]").arg(sources.join(", ")));
    writeDConfSetting("/org/gnome/desktop/input-sources/mru-sources",
                      QStringLiteral("[%1]").arg(firstSource));
}
} // namespace

void setupInputSources() {
    if (detectDesktopEnvironment() == DesktopEnvironment::GNOME) {
        setupGnomeInputSource();
        return;
    }
    LOG_DEBUG("Input source setup is available only in GNOME.\n");
}
