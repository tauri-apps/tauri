#!/usr/bin/env pwsh
echo "Building API definitions..."
cd api
yarn; yarn build
cd ..

echo "Building the Tauri CLI..."
cd cli\core
cargo build --release
cd ..\..

Set-Alias rtauri "$(pwd)\cli\core\target\release\cargo-tauri.exe"
echo "Added alias 'rtauri' for '$(pwd)\cli\core\target\release\cargo-tauri.exe'"
echo "Tauri CLI installed. Run it with '$ rtauri tauri [COMMAND]'."

$yes = New-Object System.Management.Automation.Host.ChoiceDescription "&Yes"
$no = New-Object System.Management.Automation.Host.ChoiceDescription "&No"
$options = [System.Management.Automation.Host.ChoiceDescription[]]($yes, $no)

$result = $host.ui.PromptForChoice("Node.js CLI", "Do you want to use the Node.js CLI?", $options, 1)
switch ($result) {
  0{
    cd cli\tauri.js
    yarn; yarn build
    cd ..\..
    Set-Alias stauri "$(pwd)\cli\tauri.js\bin\tauri.js"
    echo "Added alias 'stauri' for '$(pwd)\cli\tauri.js\bin\tauri.js'"
    echo "Tauri Node.js CLI installed. Run it with '$ stauri [COMMAND]'"
  }
}
