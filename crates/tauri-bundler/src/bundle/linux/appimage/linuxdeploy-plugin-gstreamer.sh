#! /bin/bash

# abort on all errors
set -e

if [ "$DEBUG" != "" ]; then
    set -x
fi

script=$(readlink -f "$0")

show_usage() {
    echo "Usage: $script --appdir <path to AppDir>"
    echo
    echo "Bundles GStreamer plugins into an AppDir"
    echo
    echo "Required variables:"
    echo "  LINUXDEPLOY=\".../linuxdeploy\" path to linuxdeploy (e.g., AppImage); set automatically when plugin is run directly by linuxdeploy"
    echo
    echo "Optional variables:"
    echo "  GSTREAMER_INCLUDE_BAD_PLUGINS=\"1\" (default: disabled; set to empty string or unset to disable)"
    echo "  GSTREAMER_PLUGINS_DIR=\"...\" (directory containing GStreamer plugins; default: guessed based on main distro architecture)"
    echo "  GSTREAMER_HELPERS_DIR=\"...\" (directory containing GStreamer helper tools like gst-plugin-scanner; default: guessed based on main distro architecture)"
    echo "  GSTREAMER_VERSION=\"1.0\" (default: 1.0)"
}

while [ "$1" != "" ]; do
    case "$1" in
        --plugin-api-version)
            echo "0"
            exit 0
            ;;
        --appdir)
            APPDIR="$2"
            shift
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo "Invalid argument: $1"
            echo
            show_usage
            exit 1
            ;;
    esac
done

if [ "$APPDIR" == "" ]; then
    show_usage
    exit 1
fi

if ! which patchelf &>/dev/null && ! type patchelf &>/dev/null; then
    echo "Error: patchelf not found"
    echo
    show_usage
    exit 2
fi

if [[ "$LINUXDEPLOY" == "" ]]; then
    echo "Error: \$LINUXDEPLOY not set"
    echo
    show_usage
    exit 3
fi

mkdir -p "$APPDIR"

export GSTREAMER_VERSION="${GSTREAMER_VERSION:-1.0}"

plugins_target_dir="$APPDIR"/usr/lib/gstreamer-"$GSTREAMER_VERSION"
helpers_target_dir="$APPDIR"/usr/lib/gstreamer"$GSTREAMER_VERSION"/gstreamer-"$GSTREAMER_VERSION"

if [ "$GSTREAMER_PLUGINS_DIR" != "" ]; then
    plugins_dir="${GSTREAMER_PLUGINS_DIR}"
elif [ -d /usr/lib/"$(uname -m)"-linux-gnu/gstreamer-"$GSTREAMER_VERSION" ]; then
    plugins_dir=/usr/lib/$(uname -m)-linux-gnu/gstreamer-"$GSTREAMER_VERSION"
else
    plugins_dir=/usr/lib/gstreamer-"$GSTREAMER_VERSION"
fi

if [ "$GSTREAMER_HELPERS_DIR" != "" ]; then
    helpers_dir="${GSTREAMER_HELPERS_DIR}"
else
    helpers_dir=/usr/lib/$(uname -m)-linux-gnu/gstreamer"$GSTREAMER_VERSION"/gstreamer-"$GSTREAMER_VERSION"
fi

if [ ! -d "$plugins_dir" ]; then
    echo "Error: could not find plugins directory: $plugins_dir"
    exit 1
fi

mkdir -p "$plugins_target_dir"

echo "Copying plugins into $plugins_target_dir"
for i in "$plugins_dir"/*; do
    [ -d "$i" ] && continue
    [ ! -f "$i" ] && echo "File does not exist: $i" && continue

    echo "Copying plugin: $i"
    cp "$i" "$plugins_target_dir"
done

"$LINUXDEPLOY" --appdir "$APPDIR"

for i in "$plugins_target_dir"/*; do
    [ -d "$i" ] && continue
    [ ! -f "$i" ] && echo "File does not exist: $i" && continue
    (file "$i" | grep -v ELF --silent) && echo "Ignoring non ELF file: $i" && continue

    echo "Manually setting rpath for $i"
    patchelf --set-rpath '$ORIGIN/..:$ORIGIN' "$i"
done

mkdir -p "$helpers_target_dir"

echo "Copying helpers in $helpers_target_dir"
for i in "$helpers_dir"/*; do
    [ -d "$i" ] && continue
    [ ! -f "$i" ] && echo "File does not exist: $i" && continue

    echo "Copying helper: $i"
    cp "$i" "$helpers_target_dir"
done

for i in "$helpers_target_dir"/*; do
    [ -d "$i" ] && continue
    [ ! -f "$i" ] && echo "File does not exist: $i" && continue
    (file "$i" | grep -v ELF --silent) && echo "Ignoring non ELF file: $i" && continue

    echo "Manually setting rpath for $i"
    patchelf --set-rpath '$ORIGIN/../..' "$i"
done

echo "Installing AppRun hook"
mkdir -p "$APPDIR"/apprun-hooks

if [ "$GSTREAMER_VERSION" == "1.0" ]; then
    cat > "$APPDIR"/apprun-hooks/linuxdeploy-plugin-gstreamer.sh <<\EOF
#! /bin/bash

export GST_REGISTRY_REUSE_PLUGIN_SCANNER="no"
export GST_PLUGIN_SYSTEM_PATH_1_0="${APPDIR}/usr/lib/gstreamer-1.0"
export GST_PLUGIN_PATH_1_0="${APPDIR}/usr/lib/gstreamer-1.0"

export GST_PLUGIN_SCANNER_1_0="${APPDIR}/usr/lib/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner"
export GST_PTP_HELPER_1_0="${APPDIR}/usr/lib/gstreamer1.0/gstreamer-1.0/gst-ptp-helper"
EOF
elif [ "$GSTREAMER_VERSION" == "0.10" ]; then
    cat > "$APPDIR"/apprun-hooks/linuxdeploy-plugin-gstreamer.sh <<\EOF
#! /bin/bash

export GST_REGISTRY_REUSE_PLUGIN_SCANNER="no"
export GST_PLUGIN_SYSTEM_PATH_0_10="${APPDIR}/usr/lib/gstreamer-1.0"

export GST_PLUGIN_SCANNER_0_10="${APPDIR}/usr/lib/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner"
export GST_PTP_HELPER_0_10="${APPDIR}/usr/lib/gstreamer1.0/gstreamer-1.0/gst-ptp-helper"
EOF
else
    echo "Warning: unknown GStreamer version: $GSTREAMER_VERSION, cannot install AppRun hook"
fi

