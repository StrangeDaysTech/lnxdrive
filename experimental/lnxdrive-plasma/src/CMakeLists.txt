set(lnxdrive_plasma_SRCS
    main.cpp
)

add_executable(lnxdrive-plasma ${lnxdrive_plasma_SRCS})

target_link_libraries(lnxdrive-plasma
    Qt6::Core
    Qt6::Gui
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::DBus
    Qt6::Widgets
    KF6::I18n
    KF6::CoreAddons
    KF6::DBusAddons
    KF6::ConfigCore
)

install(TARGETS lnxdrive-plasma ${KDE_INSTALL_TARGETS_DEFAULT_ARGS})
