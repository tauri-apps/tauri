#! /usr/bin/env bash

# GTK3 environment variables: https://docs.gtk.org/gtk3/running.html
# GTK4 environment variables: https://docs.gtk.org/gtk4/running.html

# abort on all errors
set -e

if [ "$DEBUG" != "" ]; then
    set -x
    verbose="--verbose"
fi

SCRIPT="$(basename "$(readlink -f "$0")")"

show_usage() {
    echo "Usage: $SCRIPT --appdir <path to AppDir>"
    echo
    echo "Bundles resources for applications that use GTK into an AppDir"
    echo
    echo "Required variables:"
    echo "  LINUXDEPLOY=\".../linuxdeploy\" path to linuxdeploy (e.g., AppImage); set automatically when plugin is run directly by linuxdeploy"
    #echo
    #echo "Optional variables:"
    #echo "  DEPLOY_GTK_VERSION (major version of GTK to deploy, e.g. '2', '3' or '4'; auto-detect by default)"
}

variable_is_true() {
    local var="$1"

    if [ -n "$var" ] && { [ "$var" == "true" ] || [ "$var" -gt 0 ]; } 2> /dev/null; then
        return 0 # true
    else
        return 1 # false
    fi
}

get_pkgconf_variable() {
    local variable="$1"
    local library="$2"
    local default_value="$3"

    pkgconfig_ret="$("$PKG_CONFIG" --variable="$variable" "$library")"
    if [ -n "$pkgconfig_ret" ]; then
        echo "$pkgconfig_ret"
    elif [ -n "$default_value" ]; then
        echo "$default_value"
    else
        echo "$0: there is no '$variable' variable for '$library' library." > /dev/stderr
        echo "Please check the '$library.pc' file is present in \$PKG_CONFIG_PATH (you may need to install the appropriate -dev/-devel package)." > /dev/stderr
        exit 1
    fi
}

copy_tree() {
    local src=("${@:1:$#-1}")
    local dst="${*:$#}"

    for elem in "${src[@]}"; do
        mkdir -p "${dst::-1}$elem"
        cp "$elem" --archive --parents --target-directory="$dst" $verbose
    done
}

copy_lib_tree() {
    # The source lib directory could be /usr/lib, /usr/lib64, or /usr/lib/x86_64-linux-gnu
    # Therefore, when copying lib directories, we need to transform that target path
    # to a consistent /usr/lib
    local src=("${@:1:$#-1}")
    local dst="${*:$#}"

    for elem in "${src[@]}"; do
        mkdir -p "${dst::-1}${elem/$LD_GTK_LIBRARY_PATH//usr/lib}"
        pushd "$LD_GTK_LIBRARY_PATH"
        cp "$(realpath --relative-to="$LD_GTK_LIBRARY_PATH" "$elem")" --archive --parents --target-directory="$dst/usr/lib" $verbose
        popd
    done
}

get_triplet_path() {
    if command -v dpkg-architecture > /dev/null; then
        echo "/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)"
    fi
}



search_library_path() {
    PATH_ARRAY=(
        "$(get_triplet_path)"
        "/usr/lib64"
        "/usr/lib"
    )

    for path in "${PATH_ARRAY[@]}"; do
        if [ -d "$path" ]; then
            echo "$path"
            return 0
        fi
    done
}

search_tool() {
    local tool="$1"
    local directory="$2"

    if command -v "$tool"; then
        return 0
    fi

    PATH_ARRAY=(
        "$(get_triplet_path)/$directory/$tool"
        "/usr/lib64/$directory/$tool"
        "/usr/lib/$directory/$tool"
        "/usr/bin/$tool"
        "/usr/bin/$tool-64"
        "/usr/bin/$tool-32"
    )

    for path in "${PATH_ARRAY[@]}"; do
        if [ -x "$path" ]; then
            echo "$path"
            return 0
        fi
    done
}

#DEPLOY_GTK_VERSION="${DEPLOY_GTK_VERSION:-0}" # When not set by user, this variable use the integer '0' as a sentinel value
DEPLOY_GTK_VERSION=3 # Force GTK3 for tauri apps
APPDIR=

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

APPDIR="$(realpath "$APPDIR")"
mkdir -p "$APPDIR"
# make lib64 writable again.
chmod +w "$APPDIR"/usr/lib64 || true

. /etc/os-release
if [ "$ID" = "debian" ] || [ "$ID" = "ubuntu" ]; then
    if ! command -v dpkg-architecture  &>/dev/null; then
        echo -e "$0: dpkg-architecture not found.\nInstall dpkg-dev then re-run the plugin."
        exit 1
    fi
fi

if command -v pkgconf > /dev/null; then
    PKG_CONFIG="pkgconf"
elif command -v pkg-config > /dev/null; then
    PKG_CONFIG="pkg-config"
else
    echo "$0: pkg-config/pkgconf not found in PATH, aborting"
    exit 1
fi

# GTK's library path *must not* have a trailing slash for later parameter substitution to work properly
LD_GTK_LIBRARY_PATH="$(realpath "${LD_GTK_LIBRARY_PATH:-$(search_library_path)}")"

if ! command -v find &>/dev/null && ! type find &>/dev/null; then
    echo -e "$0: find not found.\nInstall findutils then re-run the plugin."
    exit 1
fi

if [ -z "$LINUXDEPLOY" ]; then
    echo -e "$0: LINUXDEPLOY environment variable is not set.\nDownload a suitable linuxdeploy AppImage, set the environment variable and re-run the plugin."
    exit 1
fi

gtk_versions=0 # Count major versions of GTK when auto-detect GTK version
if [ "$DEPLOY_GTK_VERSION" -eq 0 ]; then
    echo "Determining which GTK version to deploy"
    while IFS= read -r -d '' file; do
        if [ "$DEPLOY_GTK_VERSION" -ne 2 ] && ldd "$file" | grep -q "libgtk-x11-2.0.so"; then
            DEPLOY_GTK_VERSION=2
            gtk_versions="$((gtk_versions+1))"
        fi
        if [ "$DEPLOY_GTK_VERSION" -ne 3 ] && ldd "$file" | grep -q "libgtk-3.so"; then
            DEPLOY_GTK_VERSION=3
            gtk_versions="$((gtk_versions+1))"
        fi
        if [ "$DEPLOY_GTK_VERSION" -ne 4 ] && ldd "$file" | grep -q "libgtk-4.so"; then
            DEPLOY_GTK_VERSION=4
            gtk_versions="$((gtk_versions+1))"
        fi
    done < <(find "$APPDIR/usr/bin" -executable -type f -print0)
fi

if [ "$gtk_versions" -gt 1 ]; then
    echo "$0: can not deploy multiple GTK versions at the same time."
    echo "Please set DEPLOY_GTK_VERSION to {2, 3, 4}."
    exit 1
elif [ "$DEPLOY_GTK_VERSION" -eq 0 ]; then
    echo "$0: failed to auto-detect GTK version."
    echo "Please set DEPLOY_GTK_VERSION to {2, 3, 4}."
    exit 1
fi

echo "Installing AppRun hook"
HOOKSDIR="$APPDIR/apprun-hooks"
HOOKFILE="$HOOKSDIR/linuxdeploy-plugin-gtk.sh"
mkdir -p "$HOOKSDIR"
cat > "$HOOKFILE" <<\EOF
#! /usr/bin/env bash

COLOR_SCHEME="$(dbus-send --session --dest=org.freedesktop.portal.Desktop --type=method_call --print-reply --reply-timeout=1000 /org/freedesktop/portal/desktop org.freedesktop.portal.Settings.Read 'string:org.freedesktop.appearance' 'string:color-scheme' 2> /dev/null | tail -n1 | cut -b35- | cut -d' ' -f2 || printf '')"
if [ -z "$COLOR_SCHEME" ]; then
    COLOR_SCHEME="$(gsettings get org.gnome.desktop.interface color-scheme 2> /dev/null || printf '')"
fi
case "$COLOR_SCHEME" in
    "1"|"'prefer-dark'")  GTK_THEME_VARIANT="dark";;
    "2"|"'prefer-light'") GTK_THEME_VARIANT="light";;
    *)                    GTK_THEME_VARIANT="light";;
esac
APPIMAGE_GTK_THEME="${APPIMAGE_GTK_THEME:-"Adwaita:$GTK_THEME_VARIANT"}" # Allow user to override theme (discouraged)

export APPDIR="${APPDIR:-"$(dirname "$(realpath "$0")")"}" # Workaround to run extracted AppImage
export GTK_DATA_PREFIX="$APPDIR"
export GTK_THEME="$APPIMAGE_GTK_THEME" # Custom themes are broken
# export GDK_BACKEND=x11 # Monitor this closely. AppImage used to crash on Wayland!
export XDG_DATA_DIRS="$APPDIR/usr/share:/usr/share:$XDG_DATA_DIRS" # g_get_system_data_dirs() from GLib
EOF

echo "Installing GLib schemas"
# Note: schemasdir is undefined on Ubuntu 16.04
glib_schemasdir="$(get_pkgconf_variable "schemasdir" "gio-2.0" "/usr/share/glib-2.0/schemas")"
copy_tree "$glib_schemasdir" "$APPDIR/"
glib-compile-schemas "$APPDIR/$glib_schemasdir"
cat >> "$HOOKFILE" <<EOF
export GSETTINGS_SCHEMA_DIR="\$APPDIR/$glib_schemasdir"
EOF

echo "Installing GIRepository Typelibs"
gi_typelibsdir="$(get_pkgconf_variable "typelibdir" "gobject-introspection-1.0" "$LD_GTK_LIBRARY_PATH/girepository-1.0")"
copy_lib_tree "$gi_typelibsdir" "$APPDIR/"
cat >> "$HOOKFILE" <<EOF
export GI_TYPELIB_PATH="\$APPDIR/${gi_typelibsdir/$LD_GTK_LIBRARY_PATH//usr/lib}"
EOF

echo "Installing Gio modules"
gio_moduledir="$(get_pkgconf_variable "giomoduledir" "gio-2.0" "$LD_GTK_LIBRARY_PATH/gio/modules")"
copy_lib_tree "$gio_moduledir" "$APPDIR/"
gio_querymodules="$(search_tool "gio-querymodules" "glib-2.0")"
if [ -x "$gio_querymodules" ]; then
    "$gio_querymodules" "$APPDIR/${gio_moduledir/$LD_GTK_LIBRARY_PATH//usr/lib}"
else
    echo "WARNING: gio-querymodules not found"
fi
cat >> "$HOOKFILE" <<EOF
export GIO_MODULE_DIR="\$APPDIR/${gio_moduledir/$LD_GTK_LIBRARY_PATH//usr/lib}"
EOF

case "$DEPLOY_GTK_VERSION" in
    2)
        # https://github.com/linuxdeploy/linuxdeploy-plugin-gtk/pull/20#issuecomment-826354261
        echo "WARNING: Gtk+2 applications are not fully supported by this plugin"
        ;;
    3)
        echo "Installing GTK 3.0 modules"
        gtk3_exec_prefix="$(get_pkgconf_variable "exec_prefix" "gtk+-3.0" "/usr")"
        gtk3_libdir="$(get_pkgconf_variable "libdir" "gtk+-3.0" "$LD_GTK_LIBRARY_PATH")/gtk-3.0"
        gtk3_path="$gtk3_libdir"
        gtk3_immodulesdir="$gtk3_libdir/$(get_pkgconf_variable "gtk_binary_version" "gtk+-3.0" "3.0.0")/immodules"
        gtk3_printbackendsdir="$gtk3_libdir/$(get_pkgconf_variable "gtk_binary_version" "gtk+-3.0" "3.0.0")/printbackends"
        gtk3_immodules_cache_file="$(dirname "$gtk3_immodulesdir")/immodules.cache"
        gtk3_immodules_query="$(search_tool "gtk-query-immodules-3.0" "libgtk-3-0")"
        copy_lib_tree "$gtk3_libdir" "$APPDIR/"
        cat >> "$HOOKFILE" <<EOF
export GTK_EXE_PREFIX="\$APPDIR/$gtk3_exec_prefix"
export GTK_PATH="\$APPDIR/${gtk3_path/$LD_GTK_LIBRARY_PATH//usr/lib}"
export GTK_IM_MODULE_FILE="\$APPDIR/${gtk3_immodules_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
EOF

        if [ -x "$gtk3_immodules_query" ]; then
            echo "Updating immodules cache in $APPDIR/${gtk3_immodules_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
            "$gtk3_immodules_query" > "$APPDIR/${gtk3_immodules_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
        else
            echo "WARNING: gtk-query-immodules-3.0 not found"
        fi
        if [ ! -f "$APPDIR/${gtk3_immodules_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}" ]; then
            echo "WARNING: immodules.cache file is missing"
        fi
        sed -i "s|$gtk3_libdir/3.0.0/immodules/||g" "$APPDIR/${gtk3_immodules_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
        ;;
    4)
        echo "Installing GTK 4.0 modules"
        gtk4_exec_prefix="$(get_pkgconf_variable "exec_prefix" "gtk4" "/usr")"
        gtk4_libdir="$(get_pkgconf_variable "libdir" "gtk4")/gtk-4.0"
        gtk4_path="$gtk4_libdir"
        copy_lib_tree "$gtk4_libdir" "$APPDIR/"
        cat >> "$HOOKFILE" <<EOF
export GTK_EXE_PREFIX="\$APPDIR/$gtk4_exec_prefix"
export GTK_PATH="\$APPDIR/${gtk4_path/$LD_GTK_LIBRARY_PATH//usr/lib}"
EOF
        ;;
    *)
        echo "$0: '$DEPLOY_GTK_VERSION' is not a valid GTK major version."
        echo "Please set DEPLOY_GTK_VERSION to {2, 3, 4}."
        exit 1
esac

echo "Installing GDK PixBufs"
gdk_libdir="$(get_pkgconf_variable "libdir" "gdk-pixbuf-2.0" "$LD_GTK_LIBRARY_PATH")"
gdk_pixbuf_binarydir="$(get_pkgconf_variable "gdk_pixbuf_binarydir" "gdk-pixbuf-2.0" "$gdk_libdir/gdk-pixbuf-2.0/2.10.0")"
gdk_pixbuf_cache_file="$(get_pkgconf_variable "gdk_pixbuf_cache_file" "gdk-pixbuf-2.0" "$gdk_pixbuf_binarydir/loaders.cache")"
gdk_pixbuf_moduledir="$(get_pkgconf_variable "gdk_pixbuf_moduledir" "gdk-pixbuf-2.0" "$gdk_pixbuf_binarydir/loaders")"
# Note: gdk_pixbuf_query_loaders variable is not defined on some systems
gdk_pixbuf_query="$(search_tool "gdk-pixbuf-query-loaders" "gdk-pixbuf-2.0")"
if [ -d "$gdk_pixbuf_binarydir" ]; then
copy_lib_tree "$gdk_pixbuf_binarydir" "$APPDIR/"
cat >> "$HOOKFILE" <<EOF
export GDK_PIXBUF_MODULE_FILE="\$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
EOF
if [ -x "$gdk_pixbuf_query" ]; then
    echo "Updating pixbuf cache in $APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
    "$gdk_pixbuf_query" > "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
else
    echo "WARNING: gdk-pixbuf-query-loaders not found"
fi
if [ ! -f "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}" ]; then
    echo "WARNING: loaders.cache file is missing"
fi
sed -i "s|$gdk_pixbuf_moduledir/||g" "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
else
    echo "WARNING: $gdk_pixbuf_binarydir not found, not bundling gdk-pixbuf loaders"
fi

echo "Copying more libraries"
gobject_libdir="$(get_pkgconf_variable "libdir" "gobject-2.0" "$LD_GTK_LIBRARY_PATH")"
gio_libdir="$(get_pkgconf_variable "libdir" "gio-2.0" "$LD_GTK_LIBRARY_PATH")"
librsvg_libdir="$(get_pkgconf_variable "libdir" "librsvg-2.0" "$LD_GTK_LIBRARY_PATH")"
pango_libdir="$(get_pkgconf_variable "libdir" "pango" "$LD_GTK_LIBRARY_PATH")"
pangocairo_libdir="$(get_pkgconf_variable "libdir" "pangocairo" "$LD_GTK_LIBRARY_PATH")"
pangoft2_libdir="$(get_pkgconf_variable "libdir" "pangoft2" "$LD_GTK_LIBRARY_PATH")"
FIND_ARRAY=(
    "$gdk_libdir"        "libgdk_pixbuf-*.so*"
    "$gobject_libdir"    "libgobject-*.so*"
    "$gio_libdir"        "libgio-*.so*"
    "$librsvg_libdir"    "librsvg-*.so*"
    "$pango_libdir"      "libpango-*.so*"
    "$pangocairo_libdir" "libpangocairo-*.so*"
    "$pangoft2_libdir"   "libpangoft2-*.so*"
)
LIBRARIES=()
for (( i=0; i<${#FIND_ARRAY[@]}; i+=2 )); do
    directory=${FIND_ARRAY[i]}
    library=${FIND_ARRAY[i+1]}
    while IFS= read -r -d '' file; do
        LIBRARIES+=( "--library=$file" )
    done < <(find "$directory" \( -type l -o -type f \) -name "$library" -print0)
done

env LINUXDEPLOY_PLUGIN_MODE=1 "$LINUXDEPLOY" --appdir="$APPDIR" "${LIBRARIES[@]}"

# Create symbolic links as a workaround
# Details: https://github.com/linuxdeploy/linuxdeploy-plugin-gtk/issues/24#issuecomment-1030026529
echo "Manually setting rpath for GTK modules"
PATCH_ARRAY=(
    "$gtk3_immodulesdir"
    "$gtk3_printbackendsdir"
    "$gdk_pixbuf_moduledir"
)
for directory in "${PATCH_ARRAY[@]}"; do
    while IFS= read -r -d '' file; do
        ln $verbose -sf "${file/$LD_GTK_LIBRARY_PATH\//}" "$APPDIR/usr/lib"
    done < <(find "$directory" -name '*.so' -print0)
done

# set write permission on lib64 again to make it deletable.
chmod +w "$APPDIR"/usr/lib64 || true


#binary patch absolute paths in libwebkit files
find "$APPDIR"/usr/lib* -name 'libwebkit*' -exec sed -i -e "s|/usr|././|g" '{}' \;
