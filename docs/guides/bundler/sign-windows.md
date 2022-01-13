---
title: Windows - Code signing guide locally & with Github Actions
sidebar_label: Windows - Code signing
---

import Alert from '@theme/Alert'

# Intro

Code-signing will add a level of authenticity to your application, while it is not required it can often improve the user experience for your users. 

*Note: I will be keeping this CLI friendly, there are ways to do this via GUI if thats your flavor, but that will not be covered in this documentation*

# Prerequisites 

- Windows - you can likely use other platforms, but this tutorial is using Powershell native features (I know, i hate this as much as you do.)
- Code signing certificate - you can aqquire one of these on services such as Digicert.com, Comodo.com, & Godaddy.com. In this guide I am using Comodo.com
- A working tauri application


# Getting Started

There are a few things we will have to do to get our windows installation prepared for code signing. This includes converting our certificate to a speific format, installing this certificate, & then decoding required information from certificate that is required by tauri.

## A. Convert your `.cer` to `.pfx`

1. You will need the following:
	- certificate file (mine is `cert.cer`) 
	- private key file (mine is `private-key.key`)

2. Open up a command prompt and change to your current directory using `cd Documents/Certs`

3. Convert your `.cer` to a `.pfx` using `openssl pkcs12 -export -in cert.cer -inkey private-key.key -out certificate.pfx`

4. You will be prompted to enter an export password **DON'T FORGET IT!**

## B. Import your `.pfx` file into the keystore. 

We will now need to import our `.pfx` file.

1. Assign your export password to a variable using `$WINDOWS_PFX_PASSWORD = 'MYPASSWORD'`

2. Now Import the certificate using `Import-PfxCertificate -FilePath Certs/certificate.pfx -CertStoreLocation Cert:\LocalMachine\My -Password (ConvertTo-SecureString -String $env:WINDOWS_PFX_PASSWORD -Force -AsPlainText)`

## C. Prepare Variables

1. We will need the SHA-1 thumbprint of the certificate, you can get this using `openssl pkcs12 -info -in certificate.pfx` and look under for following
```
Bag Attributes
    localKeyID: A1 B1 A2 B2 A3 B3 A4 B4 A5 B5 A6 B6 A7 B7 A8 B8 A9 B9 A0 B0
```

2. You will capture the `localKeyID` but with no spaces, in this example it would be `A1B1A2B2A3B3A4B4A5B5A6B6A7B7A8B8A9B9A0B0`. This is our `certificateThumbprint`.

3. We will need the SHA digest algorythm used for your certificate (Hint: this is likely `sha256`

4. We will also need a timestamp url, this is a time server used to verify the time of the certificate signing. Im using `http://timestamp.comodoca.com` but whoever you got your certificate from likely has one aswell. 

# Prepare `tauri.conf.json` file

1. Now that we have our `certificateThumbprint`, `digestAlgorithm`, & `timestampUrl` we will open up the `tauri.conf.json`.

2. In the `tauri.conf.json` you will look for the `tauri` -> `bundle` -> `windows` section. You will see there are three variable for the information we have captured. Fill it out like below. 
```
"windows": {
        "certificateThumbprint": "A1B1A2B2A3B3A4B4A5B5A6B6A7B7A8B8A9B9A0B0",
        "digestAlgorithm": "sha256",
        "timestampUrl": "http://timestamp.comodoca.com"
}
```
3. Save, and run `yarn | yarn build`

4. In the console output you will see the following output.

```
info: signing app
info: running signtool "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.19041.0\\x64\\signtool.exe"
info: "Done Adding Additional Store\r\nSuccessfully signed: APPLICATION FILE PATH HERE
``` 

which shows you have successfully signed the `.exe`. 

And thats it! You have successfully signed your .exe file.