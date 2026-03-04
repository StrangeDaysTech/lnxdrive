cmake_minimum_required(VERSION 3.16)

project(lnxdrive-plasma VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Find ECM
find_package(ECM 6.0.0 REQUIRED NO_MODULE)
set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH})

include(KDEInstallDirs)
include(KDECMakeSettings)
include(KDECompilerSettings NO_POLICY_SCOPE)
include(ECMInstallIcons)
include(ECMQtDeclareLoggingCategory)
include(FeatureSummary)

# Find Qt6
find_package(Qt6 6.6 REQUIRED COMPONENTS
    Core
    Gui
    Quick
    QuickControls2
    DBus
    Widgets
)

# Find KDE Frameworks
find_package(KF6 6.0.0 REQUIRED COMPONENTS
    I18n
    CoreAddons
    DBusAddons
    Config
    KIO
)

# Find Plasma
find_package(Plasma 6.0.0 REQUIRED)

# Subdirectories
add_subdirectory(src)
add_subdirectory(dolphin-plugin)
add_subdirectory(plasmoid)

feature_summary(WHAT ALL FATAL_ON_MISSING_REQUIRED_PACKAGES)
