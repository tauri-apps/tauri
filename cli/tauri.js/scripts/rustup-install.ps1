$client = new-object System.Net.WebClient
$client.DownloadFile('https://win.rustup.rs', "$pwd\rustup-init.exe")
.\rustup-init.exe
rm .\rustup-init.exe
