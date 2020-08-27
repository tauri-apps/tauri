#!/usr/bin/env python
"""
This script adds a license file to a DMG. Requires Xcode and a plain ascii text
license file.
Obviously only runs on a Mac.

Copyright (C) 2011-2013 Jared Hobbs

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
"""
from __future__ import print_function
import os
import sys
import tempfile
import optparse

REZ_PATH = os.popen('xcode-select --print-path', 'r').read().strip()

class Path(str):
    def __enter__(self):
        return self

    def __exit__(self, type, value, traceback):
        os.unlink(self)


def mktemp(dir=None, suffix=''):
    (fd, filename) = tempfile.mkstemp(dir=dir, suffix=suffix)
    os.close(fd)
    return Path(filename)


def main(options, args):
    dmgFile, license = args
    with mktemp('.') as tmpFile:
        with open(tmpFile, 'w') as f:
            f.write("""data 'TMPL' (128, "LPic") {
        $"1344 6566 6175 6C74 204C 616E 6775 6167"
        $"6520 4944 4457 5244 0543 6F75 6E74 4F43"
        $"4E54 042A 2A2A 2A4C 5354 430B 7379 7320"
        $"6C61 6E67 2049 4444 5752 441E 6C6F 6361"
        $"6C20 7265 7320 4944 2028 6F66 6673 6574"
        $"2066 726F 6D20 3530 3030 4457 5244 1032"
        $"2D62 7974 6520 6C61 6E67 7561 6765 3F44"
        $"5752 4404 2A2A 2A2A 4C53 5445"
};

data 'LPic' (5000) {
        $"0000 0002 0000 0000 0000 0000 0004 0000"
};

data 'STR#' (5000, "English buttons") {
        $"0006 0D45 6E67 6C69 7368 2074 6573 7431"
        $"0541 6772 6565 0844 6973 6167 7265 6505"
        $"5072 696E 7407 5361 7665 2E2E 2E7A 4966"
        $"2079 6F75 2061 6772 6565 2077 6974 6820"
        $"7468 6520 7465 726D 7320 6F66 2074 6869"
        $"7320 6C69 6365 6E73 652C 2063 6C69 636B"
        $"2022 4167 7265 6522 2074 6F20 6163 6365"
        $"7373 2074 6865 2073 6F66 7477 6172 652E"
        $"2020 4966 2079 6F75 2064 6F20 6E6F 7420"
        $"6167 7265 652C 2070 7265 7373 2022 4469"
        $"7361 6772 6565 2E22"
};

data 'STR#' (5002, "English") {
        $"0006 0745 6E67 6C69 7368 0541 6772 6565"
        $"0844 6973 6167 7265 6505 5072 696E 7407"
        $"5361 7665 2E2E 2E7B 4966 2079 6F75 2061"
        $"6772 6565 2077 6974 6820 7468 6520 7465"
        $"726D 7320 6F66 2074 6869 7320 6C69 6365"
        $"6E73 652C 2070 7265 7373 2022 4167 7265"
        $"6522 2074 6F20 696E 7374 616C 6C20 7468"
        $"6520 736F 6674 7761 7265 2E20 2049 6620"
        $"796F 7520 646F 206E 6F74 2061 6772 6565"
        $"2C20 7072 6573 7320 2244 6973 6167 7265"
        $"6522 2E"
};\n\n""")
            with open(license, 'r') as l:
                kind = 'RTF ' if license.lower().endswith('.rtf') else 'TEXT'
                f.write('data \'%s\' (5000, "English") {\n' % kind)
                def escape(s):
                    return s.strip().replace('\\', '\\\\').replace('"', '\\"').replace('\0', '')

                for line in l:
                    line = escape(line)
                    for liner in [line[i:i+1000] for i in range(0, len(line), 1000)]:
                        f.write('    "' + liner + '"\n')
                    f.write('    "' + '\\n"\n')
                f.write('};\n\n')
            f.write("""data 'styl' (5000, "English") {
        $"0003 0000 0000 000C 0009 0014 0000 0000"
        $"0000 0000 0000 0000 0027 000C 0009 0014"
        $"0100 0000 0000 0000 0000 0000 002A 000C"
        $"0009 0014 0000 0000 0000 0000 0000"
};\n""")
        os.system('hdiutil unflatten -quiet "%s"' % dmgFile)
        ret = os.system('%s -a %s -o "%s"' %
                        (options.rez, tmpFile, dmgFile))
        os.system('hdiutil flatten -quiet "%s"' % dmgFile)
        if options.compression is not None:
            os.system('cp %s %s.temp.dmg' % (dmgFile, dmgFile))
            os.remove(dmgFile)
            if options.compression == "bz2":
                os.system('hdiutil convert %s.temp.dmg -format UDBZ -o %s' %
                          (dmgFile, dmgFile))
            elif options.compression == "gz":
                os.system('hdiutil convert %s.temp.dmg -format ' % dmgFile +
                          'UDZO -imagekey zlib-devel=9 -o %s' % dmgFile)
            os.remove('%s.temp.dmg' % dmgFile)
    if ret == 0:
        print("Successfully added license to '%s'" % dmgFile)
    else:
        print("Failed to add license to '%s'" % dmgFile)

if __name__ == '__main__':
    parser = optparse.OptionParser()
    parser.set_usage("""%prog <dmgFile> <licenseFile> [OPTIONS]
  This program adds a software license agreement to a DMG file.
  It requires Xcode and either a plain ascii text <licenseFile>
  or a <licenseFile.rtf> with the RTF contents.

  See --help for more details.""")
    parser.add_option(
        '--rez',
        '-r',
        action='store',
        default=REZ_PATH,
        help='The path to the Rez tool. Defaults to %default'
    )
    parser.add_option(
        '--compression',
        '-c',
        action='store',
        choices=['bz2', 'gz'],
        default=None,
        help='Optionally compress dmg using specified compression type. '
             'Choices are bz2 and gz.'
    )
    options, args = parser.parse_args()
    cond = len(args) != 2
    if not os.path.exists(options.rez):
        print('Failed to find Rez at "%s"!\n' % options.rez)
        cond = True
    if cond:
        parser.print_usage()
        sys.exit(1)
    main(options, args)
