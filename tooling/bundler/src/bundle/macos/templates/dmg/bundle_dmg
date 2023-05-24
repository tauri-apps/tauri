#!/usr/bin/env bash
# Copyright 2019-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

# Create a read-only disk image of the contents of a folder
# forked from https://github.com/andreyvit/create-dmg

# Bail out on any unhandled errors
set -ex;

function pure_version() {
	echo '1.0.0.6'
}

function version() {
	echo "create-dmg $(pure_version)"
}

function usage() {
	version
	echo "Creates a fancy DMG file."
	echo "Usage:  $(basename "$0") [options] <output_name.dmg> <source_folder>"
	echo "All contents of <source_folder> will be copied into the disk image."
	echo "Options:"
	echo "  --volname name"
	echo "      set volume name (displayed in the Finder sidebar and window title)"
	echo "  --volicon icon.icns"
	echo "      set volume icon"
	echo "  --background pic.png"
	echo "      set folder background image (provide png, gif, jpg)"
	echo "  --window-pos x y"
	echo "      set position the folder window"
	echo "  --window-size width height"
	echo "      set size of the folder window"
	echo "  --text-size text_size"
	echo "      set window text size (10-16)"
	echo "  --icon-size icon_size"
	echo "      set window icons size (up to 128)"
	echo "  --icon file_name x y"
	echo "      set position of the file's icon"
	echo "  --hide-extension file_name"
	echo "      hide the extension of file"
	echo "  --app-drop-link x y"
	echo "      make a drop link to Applications, at location x,y"
	echo "  --ql-drop-link x y"
	echo "      make a drop link to user QuickLook install dir, at location x,y"
	echo "  --eula eula_file"
	echo "      attach a license file to the dmg"
	echo "  --no-internet-enable"
	echo "      disable automatic mount&copy"
	echo "  --format"
	echo "      specify the final image format (default is UDZO)"
	echo "  --add-file target_name file|folder x y"
	echo "      add additional file or folder (can be used multiple times)"
	echo "  --disk-image-size x"
	echo "      set the disk image size manually to x MB"
	echo "  --hdiutil-verbose"
	echo "      execute hdiutil in verbose mode"
	echo "  --hdiutil-quiet"
	echo "      execute hdiutil in quiet mode"
	echo "  --bless"
  echo "      bless the mount folder (deprecated, needs macOS 12.2.1 or older)"
	echo "  --sandbox-safe"
	echo "      execute hdiutil with sandbox compatibility, do not bless and do not execute the cosmetic AppleScript"
	echo "  --version         show tool version number"
	echo "  -h, --help        display this help"
	exit 0
}

WINX=10
WINY=60
WINW=500
WINH=350
ICON_SIZE=128
TEXT_SIZE=16
FORMAT="UDZO"
ADD_FILE_SOURCES=()
ADD_FILE_TARGETS=()
IMAGEKEY=""
HDIUTIL_VERBOSITY=""
SANDBOX_SAFE=0
BLESS=0
SKIP_JENKINS=0
MAXIMUM_UNMOUNTING_ATTEMPTS=3
POSITION_CLAUSE=""
HIDING_CLAUSE=""

while [[ "${1:0:1}" = "-" ]]; do
	case $1 in
	--volname)
		VOLUME_NAME="$2"
		shift; shift;;
	--volicon)
		VOLUME_ICON_FILE="$2"
		shift; shift;;
	--background)
		BACKGROUND_FILE="$2"
		BACKGROUND_FILE_NAME="$(basename "$BACKGROUND_FILE")"
		BACKGROUND_CLAUSE="set background picture of opts to file \".background:$BACKGROUND_FILE_NAME\""
		REPOSITION_HIDDEN_FILES_CLAUSE="set position of every item to {theBottomRightX + 100, 100}"
		shift; shift;;
	--icon-size)
		ICON_SIZE="$2"
		shift; shift;;
	--text-size)
		TEXT_SIZE="$2"
		shift; shift;;
	--window-pos)
		WINX=$2; WINY=$3
		shift; shift; shift;;
	--window-size)
		WINW=$2; WINH=$3
		shift; shift; shift;;
	--icon)
		POSITION_CLAUSE="${POSITION_CLAUSE}set position of item \"$2\" to {$3, $4}
		"
		shift; shift; shift; shift;;
	--hide-extension)
		HIDING_CLAUSE="${HIDING_CLAUSE}set the extension hidden of item \"$2\" to true
		"
		shift; shift;;
	-h | --help)
		usage;;
	--version)
		version; exit 0;;
	--pure-version)
		pure_version; exit 0;;
	--ql-drop-link)
		QL_LINK=$2
		QL_CLAUSE="set position of item \"QuickLook\" to {$2, $3}
		"
		shift; shift; shift;;
	--app-drop-link)
		APPLICATION_LINK=$2
		APPLICATION_CLAUSE="set position of item \"Applications\" to {$2, $3}
		"
		shift; shift; shift;;
	--eula)
		EULA_RSRC=$2
		shift; shift;;
	--no-internet-enable)
		NOINTERNET=1
		shift;;
	--format)
		FORMAT="$2"
		shift; shift;;
	--add-file | --add-folder)
		ADD_FILE_TARGETS+=("$2")
		ADD_FILE_SOURCES+=("$3")
		POSITION_CLAUSE="${POSITION_CLAUSE}
		set position of item \"$2\" to {$4, $5}
		"
		shift; shift; shift; shift; shift;;
	--disk-image-size)
		DISK_IMAGE_SIZE="$2"
		shift; shift;;
	--hdiutil-verbose)
		HDIUTIL_VERBOSITY='-verbose'
		shift;;
	--hdiutil-quiet)
		HDIUTIL_VERBOSITY='-quiet'
		shift;;
	--sandbox-safe)
		SANDBOX_SAFE=1
		shift;;
	--bless)
		BLESS=1
		shift;;
	--skip-jenkins)
		SKIP_JENKINS=1
		shift;;
	-*)
		echo "Unknown option: $1. Run with --help for help."
		exit 1;;
	esac
	case $FORMAT in
	UDZO)
		IMAGEKEY="-imagekey zlib-level=9";;
	UDBZ)
		IMAGEKEY="-imagekey bzip2-level=9";;
	esac
done

if [[ -z "$2" ]]; then
	echo "Not enough arguments. Invoke with --help for help."
	exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DMG_PATH="$1"
DMG_DIRNAME="$(dirname "$DMG_PATH")"
DMG_DIR="$(cd "$DMG_DIRNAME" > /dev/null; pwd)"
DMG_NAME="$(basename "$DMG_PATH")"
DMG_TEMP_NAME="$DMG_DIR/rw.${DMG_NAME}"
SRC_FOLDER="$(cd "$2" > /dev/null; pwd)"

# Argument validation checks

if [[ "${DMG_PATH: -4}" != ".dmg" ]]; then
	echo "Output file name must end with a .dmg extension. Run 'create-dmg --help' for help."
	exit 1
fi

if [[ -z "$VOLUME_NAME" ]]; then
	VOLUME_NAME="$(basename "$DMG_PATH" .dmg)"
fi

# brew formula will set this as 1 and embed the support scripts
BREW_INSTALL=0

AUX_PATH="$SCRIPT_DIR/support"

if [ $BREW_INSTALL -eq 0 ]; then
	test -d "$AUX_PATH" || {
		echo "Cannot find support directory: $AUX_PATH"
		exit 1
	}
fi

if [[ -f "$SRC_FOLDER/.DS_Store" ]]; then
	echo "Deleting any .DS_Store in source folder"
	rm "$SRC_FOLDER/.DS_Store"
fi

# Create the image
echo "Creating disk image..."
if [[ -f "${DMG_TEMP_NAME}" ]]; then
	rm -f "${DMG_TEMP_NAME}"
fi

# Using Megabytes since hdiutil fails with very large Byte numbers
function blocks_to_megabytes() {
	# Add 1 extra MB, since there's no decimal retention here
	MB_SIZE=$((($1 * 512 / 1000 / 1000) + 1))
	echo $MB_SIZE
}

function get_size() {
	# Get block size in disk
	bytes_size=$(du -s "$1" | sed -e 's/	.*//g')
	echo $(blocks_to_megabytes "$bytes_size")
}

# Create the DMG with the specified size or the hdiutil estimation
CUSTOM_SIZE=''
if [[ -n "$DISK_IMAGE_SIZE" ]]; then
	CUSTOM_SIZE="-size ${DISK_IMAGE_SIZE}m"
fi

if [ $SANDBOX_SAFE -eq 0 ]; then
	hdiutil create ${HDIUTIL_VERBOSITY} -srcfolder "$SRC_FOLDER" -volname "${VOLUME_NAME}" -fs HFS+ -fsargs "-c c=64,a=16,e=16" -format UDRW ${CUSTOM_SIZE} "${DMG_TEMP_NAME}"
else
	hdiutil makehybrid ${HDIUTIL_VERBOSITY} -default-volume-name "${VOLUME_NAME}" -hfs -o "${DMG_TEMP_NAME}" "$SRC_FOLDER"
	hdiutil convert -format UDRW -ov -o "${DMG_TEMP_NAME}" "${DMG_TEMP_NAME}"
	DISK_IMAGE_SIZE_CUSTOM=$DISK_IMAGE_SIZE
fi

# Get the created DMG actual size
DISK_IMAGE_SIZE=$(get_size "${DMG_TEMP_NAME}")

# Use the custom size if bigger
if [[ $SANDBOX_SAFE -eq 1 ]] && [[ -n "$DISK_IMAGE_SIZE_CUSTOM" ]] && [[ $DISK_IMAGE_SIZE_CUSTOM -gt $DISK_IMAGE_SIZE ]]; then
	DISK_IMAGE_SIZE=$DISK_IMAGE_SIZE_CUSTOM
fi

# Estimate the additional sources size
if [[ -n "$ADD_FILE_SOURCES" ]]; then
	for i in "${!ADD_FILE_SOURCES[@]}"; do
		SOURCE_SIZE=$(get_size "${ADD_FILE_SOURCES[$i]}")
		DISK_IMAGE_SIZE=$(expr $DISK_IMAGE_SIZE + $SOURCE_SIZE)
	done
fi

# Add extra space for additional resources
DISK_IMAGE_SIZE=$(expr $DISK_IMAGE_SIZE + 20)

# Make sure target image size is within limits
MIN_DISK_IMAGE_SIZE=$(hdiutil resize -limits "${DMG_TEMP_NAME}" | awk 'NR=1{print int($1/2048+1)}')
if [ $MIN_DISK_IMAGE_SIZE -gt $DISK_IMAGE_SIZE ]; then
  DISK_IMAGE_SIZE=$MIN_DISK_IMAGE_SIZE
fi

# Resize the image for the extra stuff
hdiutil resize ${HDIUTIL_VERBOSITY} -size ${DISK_IMAGE_SIZE}m "${DMG_TEMP_NAME}"

# mount the new DMG
echo "Mounting disk image..."
MOUNT_DIR="/Volumes/${VOLUME_NAME}"

# Unmount leftover dmg if it was mounted previously (e.g. developer mounted dmg, installed app and forgot to unmount it)
if [[ -d "${MOUNT_DIR}" ]]; then
  echo "Unmounting previously mounted disk image..."
	DEV_NAME=$(hdiutil info | grep -E --color=never '^/dev/' | sed 1q | awk '{print $1}')
  hdiutil detach "${DEV_NAME}"
fi

echo "Mounting disk image..."

echo "Mount directory: $MOUNT_DIR"
DEV_NAME=$(hdiutil attach -readwrite -noverify -noautoopen "${DMG_TEMP_NAME}" | grep -E --color=never '^/dev/' | sed 1q | awk '{print $1}')
echo "Device name:     $DEV_NAME"

if [[ -n "$BACKGROUND_FILE" ]]; then
	echo "Copying background file '$BACKGROUND_FILE'..."
	[[ -d "$MOUNT_DIR/.background" ]] || mkdir "$MOUNT_DIR/.background"
	cp "$BACKGROUND_FILE" "$MOUNT_DIR/.background/$BACKGROUND_FILE_NAME"
fi

if [[ -n "$APPLICATION_LINK" ]]; then
	echo "making link to Applications dir"
	test -d "$MOUNT_DIR/Applications" || ln -s /Applications "$MOUNT_DIR/Applications"
fi

if [[ -n "$QL_LINK" ]]; then
	echo "making link to QuickLook install dir"
	ln -s "/Library/QuickLook" "$MOUNT_DIR/QuickLook"
fi

if [[ -n "$VOLUME_ICON_FILE" ]]; then
	echo "Copying volume icon file '$VOLUME_ICON_FILE'..."
	cp "$VOLUME_ICON_FILE" "$MOUNT_DIR/.VolumeIcon.icns"
	SetFile -c icnC "$MOUNT_DIR/.VolumeIcon.icns"
fi

if [[ -n "$ADD_FILE_SOURCES" ]]; then
	echo "Copying custom files..."
	for i in "${!ADD_FILE_SOURCES[@]}"; do
		echo "${ADD_FILE_SOURCES[$i]}"
		cp -a "${ADD_FILE_SOURCES[$i]}" "$MOUNT_DIR/${ADD_FILE_TARGETS[$i]}"
	done
fi

# run AppleScript to do all the Finder cosmetic stuff
APPLESCRIPT_FILE=$(mktemp -t createdmg.tmp.XXXXXXXXXX)

function applescript_source() {
	if [ $BREW_INSTALL -eq 0 ]; then
		cat "$AUX_PATH/template.applescript"
	else
		cat << 'EOS'
		# BREW_INLINE_APPLESCRIPT_PLACEHOLDER
EOS
	fi
}

if [[ $SANDBOX_SAFE -eq 1 ]]; then
	echo "Skipping Finder-prettifying AppleScript because we are in Sandbox..."
else
	if [[ $SKIP_JENKINS -eq 0 ]]; then
		applescript_source \
			| sed -e "s/WINX/$WINX/g" -e "s/WINY/$WINY/g" -e "s/WINW/$WINW/g" \
					-e "s/WINH/$WINH/g" -e "s/BACKGROUND_CLAUSE/$BACKGROUND_CLAUSE/g" \
					-e "s/REPOSITION_HIDDEN_FILES_CLAUSE/$REPOSITION_HIDDEN_FILES_CLAUSE/g" \
					-e "s/ICON_SIZE/$ICON_SIZE/g" -e "s/TEXT_SIZE/$TEXT_SIZE/g" \
			| perl -pe "s/POSITION_CLAUSE/$POSITION_CLAUSE/g" \
			| perl -pe "s/QL_CLAUSE/$QL_CLAUSE/g" \
			| perl -pe "s/APPLICATION_CLAUSE/$APPLICATION_CLAUSE/g" \
			| perl -pe "s/HIDING_CLAUSE/$HIDING_CLAUSE/" \
			> "$APPLESCRIPT_FILE"
		sleep 2 # pause to workaround occasional "Can’t get disk" (-1728) issues
		echo "Running AppleScript to make Finder stuff pretty: /usr/bin/osascript \"${APPLESCRIPT_FILE}\" \"${VOLUME_NAME}\""
		if /usr/bin/osascript "${APPLESCRIPT_FILE}" "${VOLUME_NAME}"; then
			# Okay, we're cool
			true
		else
			echo >&2 "Failed running AppleScript"
			hdiutil detach "${DEV_NAME}"
			exit 64
		fi
		echo "Done running the AppleScript..."
		sleep 4
		rm "$APPLESCRIPT_FILE"
	fi
fi

# make sure it's not world writeable
echo "Fixing permissions..."
chmod -Rf go-w "${MOUNT_DIR}" &> /dev/null || true
echo "Done fixing permissions."

# make the top window open itself on mount:
if [[ $BLESS -eq 1 && $SANDBOX_SAFE -eq 0 ]]; then
	echo "Blessing started"
	bless --folder "${MOUNT_DIR}" --openfolder "${MOUNT_DIR}"
	echo "Blessing finished"
else
	echo "Skipping blessing on sandbox"
fi

if [[ -n "$VOLUME_ICON_FILE" ]]; then
	# tell the volume that it has a special file attribute
	SetFile -a C "$MOUNT_DIR"
fi

# Delete unnecessary file system events log
echo "Deleting .fseventsd"
rm -rf "${MOUNT_DIR}/.fseventsd"

# unmount
unmounting_attempts=0
until
  echo "Unmounting disk image..."
  (( unmounting_attempts++ ))
  hdiutil detach "${DEV_NAME}"
	exit_code=$?
	(( exit_code ==  0 )) && break            # nothing goes wrong
	(( exit_code != 16 )) && exit $exit_code  # exit with the original exit code
	# The above statement returns 1 if test failed (exit_code == 16).
	#   It can make the code in the {do... done} block to be executed
do
  (( unmounting_attempts == MAXIMUM_UNMOUNTING_ATTEMPTS )) && exit 16  # patience exhausted, exit with code EBUSY
	echo "Wait a moment..."
  sleep $(( 1 * (2 ** unmounting_attempts) ))
done
unset unmounting_attempts

# compress image
echo "Compressing disk image..."
hdiutil convert ${HDIUTIL_VERBOSITY} "${DMG_TEMP_NAME}" -format "${FORMAT}" ${IMAGEKEY} -o "${DMG_DIR}/${DMG_NAME}"
rm -f "${DMG_TEMP_NAME}"

# adding EULA resources
if [[ -n "${EULA_RSRC}" && "${EULA_RSRC}" != "-null-" ]]; then
	echo "adding EULA resources"
	#
	# Use udifrez instead flatten/rez/unflatten
	# https://github.com/create-dmg/create-dmg/issues/109
	#
	# Based on a thread from dawn2dusk & peterguy
	# https://developer.apple.com/forums/thread/668084
	#
	EULA_RESOURCES_FILE=$(mktemp -t createdmg.tmp.XXXXXXXXXX)
	EULA_FORMAT=$(file -b "${EULA_RSRC}")
	if [[ ${EULA_FORMAT} == 'Rich Text Format data'* ]] ; then
		EULA_FORMAT='RTF '
	else
		EULA_FORMAT='TEXT'
	fi

	# Encode the EULA to base64
	# Replace 'openssl base64' with 'base64' if Mac OS X 10.6 support is no more needed
	# EULA_DATA="$(base64 -b 52 "${EULA_RSRC}" | sed s$'/^\(.*\)$/\t\t\t\\1/')"
	EULA_DATA="$(openssl base64 -in "${EULA_RSRC}" | tr -d '\n' | awk '{gsub(/.{52}/,"&\n")}1' | sed s$'/^\(.*\)$/\t\t\t\\1/')"
	# Fill the template with the custom EULA contents
	eval "cat > \"${EULA_RESOURCES_FILE}\" <<EOF
	$(<${AUX_PATH}/eula-resources-template.xml)
	EOF
	"
	# Apply the resources
	hdiutil udifrez -xml "${EULA_RESOURCES_FILE}" '' -quiet "${DMG_DIR}/${DMG_NAME}" || {
		echo "Failed to add the EULA license"
		exit 1
	}
	echo "Successfully added the EULA license"
fi

if [[ -n "${NOINTERNET}" && "${NOINTERNET}" == 1 ]]; then
	echo "not setting 'internet-enable' on the dmg"
else
	# check if hdiutil supports internet-enable
	# support was removed in macOS 10.15
	# https://github.com/andreyvit/create-dmg/issues/76
	if hdiutil internet-enable -help >/dev/null 2>/dev/null
	then
		hdiutil internet-enable -yes "${DMG_DIR}/${DMG_NAME}"
	else
		echo "hdiutil does not support internet-enable. Note it was removed in macOS 10.15."
	fi
fi

echo "Disk image done"
exit 0
