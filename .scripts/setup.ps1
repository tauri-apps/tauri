#!/usr/bin/env pwsh
echo "Building API definitions..."
cd api
yarn; yarn build
cd ..

echo "Installing the Tauri Rust CLI..."
cd cli\core
cargo install --path .
cd ..\..
echo "Tauri Rust CLI installed. Run it with '$ cargo tauri [COMMAND]'."

$yes = New-Object System.Management.Automation.Host.ChoiceDescription "&Yes"
$no = New-Object System.Management.Automation.Host.ChoiceDescription "&No"
$options = [System.Management.Automation.Host.ChoiceDescription[]]($yes, $no)

$result = $host.ui.PromptForChoice("Node.js CLI", "Do you want to install the Node.js CLI?", $options, 1)
switch ($result) {
  0{
    cd cli\tauri.js
    yarn; yarn build; yarn link
    cd ..\..
    echo "Tauri Node.js CLI installed. Run it with '$ stauri [COMMAND]'"
  }
}
