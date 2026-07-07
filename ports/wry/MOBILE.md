# Mobile Setup for Wry

We use [cargo-mobile2](https://github.com/tauri-apps/cargo-mobile2) to create a mobile project for both Xcode and Android studio.

## Prerequisite

- Works on **Linux**, **Windows**, **macOS**, and **WSL**(Windows subsystem for Linux).
- **Xcode** and [**Android Studio**](https://developer.android.com/studio) installed properly. This is **the most difficult** part IMHO. This means all toolchains and SDK are all installed. Please report an issue with **comprehensive** steps if you encounter any problem.

## Setting up Android Environment

### 1. Installing JDK

#### Using Android Studio:

If you have Android Studio installed, it ships with a version of JDK so you don't have to install it manually. It is usually at `<path of android studio installation>/jre`. It will be used for `JAVA_HOME` env var.

> On macOS, it can be found at `/Applications/Android\ Studio.app/Contents/jbr/Contents/Home`
> On Windows, it can be found at `C:\Program Files\Android\Android Studio\jre`

#### Using the terminal:

##### Linux/WSL

- Install it by running the following command based on your distro to install JDK:

  - debian-based
    ```
    sudo apt install default-jdk
    ```
  - arch-based
    ```
    sudo pacman -S jdk-openjdk
    ```

- Set the `JAVA_HOME` env variable for this current shell (we will make it permanent later on)
  ```bash
  export JAVA_HOME="/usr/lib/jvm/java-11-openjdk-amd64"
  ```

#### macOS

- Install openjdk from Homebrew:

```
brew install openjdk
```

- Link to system Java wrapper and set the `JAVA_HOME` env:

```
sudo ln -sfn /opt/homebrew/opt/openjdk/libexec/openjdk.jdk /Library/Java/JavaVirtualMachines/openjdk.jdk
export JAVA_HOME="/Library/Java/JavaVirtualMachines/openjdk.jdk/Contents/Home"
```

##### Windows

- Download openjdk-11
  ```powershell
  cd $HOME\downloads
  Invoke-WebRequest https://download.java.net/java/GA/jdk11/9/GPL/openjdk-11.0.2_windows-x64_bin.zip -o openjdk-11.zip
  Expand-Archive openjdk-11.zip -d .
  mkdir $env:LocalAppData\Java
  mv jdk-11.0.2 $env:LocalAppData\Java
  ```
- Set the `JAVA_HOME` env variable for this current shell (we will make it permanent later on)
  ```powershell
  $env:JAVA_HOME="$env:LocalAppData\Java\jdk-11.0.2"
  ```

### 2. Installing Android SDK and NDK

There are two ways to install the sdk and ndk.

#### Using Android Studio:

You can use the SDK Manager in Android Studio to install:

1. Android Sdk Platform 33
2. Android SDK Platform-Tools
3. NDK (Side by side) 25.0.8775105
4. Android SDK Build-Tools 33.0.
5. Android SDK Command-line Tools

> Note: you may need to tick `Show Package Details` in the right bottom corner to be able to see some of these components

#### Using the terminal:

If you don't want or can't use Android Studio you can still get the SDK Manager cli quite easily and use it to install other components.

> Note: The SDK Manager is part of the "Command line tools only" that can be downloaded from [here](https://developer.android.com/studio#command-tools)

##### Linux/WSL/macOS

Download the `cmdline-tools`

```bash
cd ~/Downloads

# if you are on Linux/WSL:
wget https://dl.google.com/android/repository/commandlinetools-linux-8512546_latest.zip -O
# if you are on macos:
wget https://dl.google.com/android/repository/commandlinetools-mac-8512546_latest.zip -O

unzip cmdline-tools.zip
cd cmdline-tools
mkdir latest
mv bin latest/
mv lib latest/
mv NOTICE.txt latest/
mv source.properties latest/
cd ..
mkdir ~/.android # You can use another location for your SDK but I prefer using ~/.android
mv cmdline-tools ~/.android
```

Install required SDK and NDK components

```bash
export ANDROID_HOME="$HOME/.android"
~/.android/cmdline-tools/latest/bin/sdkmanager "platforms;android-33" "platform-tools" "ndk;25.0.8775105" "build-tools;33.0.0"
# Install the emulator if you plan on using a virtual device later
~/.android/cmdline-tools/latest/bin/sdkmanager "emulator"
```

##### Windows

Download the `cmdline-tools`

```powershell
cd $HOME\downloads
Invoke-WebRequest https://dl.google.com/android/repository/commandlinetools-win-8512546_latest.zip -o cmdline-tools.zip
Expand-Archive cmdline-tools.zip -d .
cd cmdline-tools
mkdir latest
mv bin latest/
mv lib latest/
mv NOTICE.txt latest/
mv source.properties latest/
cd ..
mkdir $HOME\.android # You can use another location for your SDK but I prefer using $HOME\.android
mv cmdline-tools $HOME\.android
```

Install required SDK and NDK components

```powershell
$env:ANDROID_HOME="$HOME\.android"
&"$env:ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.exe" "platforms;android-33" "platform-tools" "ndk;25.0.8775105" "build-tools;33.0.0"
# Install the emulator if you plan on using a virtual device later
&"$env:ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.exe" "emulator"
```

> Note: the location you moved the `cmdline-tools` directory into will be the location of your android SDK.

### 3. Setting up Environment Variables

You'll need to set up some environment variables to get everything to work properly. The environment variables below should be all the ones your need to be able to use [cargo-mobile2](https://github.com/tauri-apps/cargo-mobile2) to build/run your android app.

##### Linux/WSL/macOS

- Setting `JAVA_HOME`:

```bash
# In .bashrc or .zshrc:
export JAVA_HOME="/usr/lib/jvm/java-11-openjdk-amd64"
# If you are using Android studio, on Linux, it is:
export JAVA_HOME=/opt/android-studio/jre
# And on macOS, it is:
export JAVA_HOME=/Applications/Android\ Studio.app/Contents/jbr/Contents/Home
```

- Setting `ANDROID_HOME`:

```bash
export ANDROID_HOME="$HOME/.android"
# If you are using Android studio, on Linux, it is:
export ANDROID_HOME="$HOME/Android/Sdk"
# And on macOS, it is:
export ANDROID_HOME="$HOME/Library/Android/sdk"
```

- Setting `PATH`:

```bash
export NDK_HOME="$ANDROID_HOME/ndk/25.0.8775105" # The patch version might be different
export PATH="$PATH:$ANDROID_HOME/cmdline-tools/latest/bin"
export PATH="$PATH:$ANDROID_HOME/platform-tools"
```

> For WSL:
> you also need to get ADB to connect to your emulator that is running on Windows
>
> ```bash
> export WSL_HOST="192.168.1.2" # Run `ipconfig` in windows to get your computer IP
> export ADB_SERVER_SOCKET=tcp:$WSL_HOST:5037
> ```

After updating `.bashrc` either run `source ~/.bashrc` or reopen your terminal to apply the changes.

##### Windows

Open a powershell instance and run the following commands in order

```powershell
Function Add-EnvVar($name, $value) { [System.Environment]::SetEnvironmentVariable("$name", "$value", "User") }
Function Add-PATHEntry($path) { $newPath = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + $path; [System.Environment]::SetEnvironmentVariable("Path", "$newPath", "User") }

Add-EnvVar JAVA_HOME "$env:LocalAppData\Java\jdk-11.0.2" # if you are using Android studio, the location is different, see the section above about JDK
$env:SDK_ROOT="$HOME\.android"# if you are using Android studio, the sdk location will be at `$env:LocalAppData\Android\Sdk`
Add-EnvVar ANDROID_HOME "$env:SDK_ROOT"
Add-EnvVar NDK_HOME "$env:SDK_ROOT\ndk\25.0.8775105"

Add-PATHEntry "$env:SDK_ROOT\cmdline-tools\latest\bin"
Add-PATHEntry "$env:SDK_ROOT\platform-tools"
```

> IMPORTANT: you need to reboot your Windows machine in order for the environement variables to be loaded correctly.

You should now have all the environment variables required and the cmdline-tools available in your PATH. You can verify this by running `sdkmanager` which should now be showing its help info.

### 4. Install Rust android targets:

```shell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

## Getting Started

Now lets bootstrap a project to develop a tauri or wry project for mobile.

- Install [cargo-mobile2](https://github.com/tauri-apps/cargo-mobile2) CLI by running:
  ```bash
  cargo install --git  https://github.com/tauri-apps/cargo-mobile2
  ```
- Create a directory and init the project.
  ```bash
  mkdir hello
  cd hello
  cargo mobile init
  # Project name (hello):
  # Stylized name (Hello):
  # Domain (example.com): tauri.app
  # Detected template packs:
  #   [0] bevy
  #   [1] bevy-demo
  #   [2] wgpu
  #   [3] winit
  #   [4] wry
  #   Enter an index for a template pack above.
  # Template pack (0): 4
  ```

## Build and Run on Device

### Android

> Make sure you're device is connected to adb
> you can check by running `cargo android list` or `adb devices`

- `cargo android run`

### iOS

- `cargo build --target aarch64-apple-ios`
- `cargo apple run`

First time running the app will be blocked. Go to your phone's `Settings > Privacy & Security > Developer Mode` to enable developer mode. And then go to `Settings -> General -> VPN and device management -> From "Developer App"` section to press "Apple Development: APPLE_ID" -> Trust.

## Build and Run on Emulator

### Android

##### Using Android Studio

- Open the project in Android Studio `cargo android open`
- Click `Trust Project`, `Use Embedded JDK`
- Choose an emulator. I usually choose Pixel 4 API 32
- (optional) if you face this error `Device supports x86, but APK only supports armeabi-v7a` then check this [Stack Overflow answer](https://stackoverflow.com/questions/41775988/what-is-the-reason-for-the-error-device-supports-x86-but-apk-only-supports-arm/43742161#43742161) to fix it.
- Press run button.

##### Without Android Studio

If you don't have access to Android Studio or don't want or when running in WSL, you can build and run the generated project directly from the terminal

1. List available emulators
   - Linux/WSL/macOS:
     ```bash
     $ANDROID_HOME/emulator/emulator -list-avds
     ```
   - Windows:
     ```powershell
     &"$env:ANDROID_HOME\emulator\emulator" -list-avds
     ```
     you should now see a list of available emulators like the following, you'll need one of them for the next step:
   ```
   Resizable_API_33
   Pixel_5_API_33
   ```
2. Start the emulator with the name of the desired emulator:
   - Linux/WSL/macOS:
     ```bash
     $ANDROID_HOME/emulator/emulator -avd Resizable_API_33
     ```
   - Windows:
     ```powershell
      &"$env:ANDROID_HOME\emulator\emulator" -avd Resizable_API_33
     ```
3. In a new terminal window, run:
   ```bash
   cargo android run
   ```

### iOS

- If you are on x86_64: `cargo build --target x86_64-apple-ios`
- If you are on M1: `cargo build --target aarch64-apple-ios-sim`
- `cargo apple open`
- Choose a simulator.
- Press run button.

## Devtools

Set `devtools` attribute to true when building webview.

### Android

Open `chrome://inspect/#devices` in Chrome to get the devtools window.

### iOS

Open Safari > Develop > [Your Device Name] > [Your WebView].
