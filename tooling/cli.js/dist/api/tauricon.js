(function webpackUniversalModuleDefinition(root, factory) {
	if(typeof exports === 'object' && typeof module === 'object')
		module.exports = factory();
	else if(typeof define === 'function' && define.amd)
		define([], factory);
	else if(typeof exports === 'object')
		exports["tauri"] = factory();
	else
		root["tauri"] = factory();
})(this, function() {
return /******/ (() => { // webpackBootstrap
/******/ 	"use strict";
/******/ 	var __webpack_modules__ = ({

/***/ "./node_modules/imagemin/index.js":
/*!****************************************!*\
  !*** ./node_modules/imagemin/index.js ***!
  \****************************************/
/***/ ((__unused_webpack___webpack_module__, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "default": () => (/* binding */ imagemin)
/* harmony export */ });
/* harmony import */ var util__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! util */ "util");
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! path */ "path");
/* harmony import */ var graceful_fs__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! graceful-fs */ "graceful-fs");
/* harmony import */ var fs__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! fs */ "fs");
/* harmony import */ var file_type__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! file-type */ "./node_modules/imagemin/node_modules/file-type/index.js");
/* harmony import */ var globby__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! globby */ "globby");
/* harmony import */ var p_pipe__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! p-pipe */ "./node_modules/p-pipe/index.js");
/* harmony import */ var replace_ext__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! replace-ext */ "replace-ext");
/* harmony import */ var junk__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! junk */ "junk");










const readFile = (0,util__WEBPACK_IMPORTED_MODULE_0__.promisify)(graceful_fs__WEBPACK_IMPORTED_MODULE_2__.readFile);
const writeFile = (0,util__WEBPACK_IMPORTED_MODULE_0__.promisify)(graceful_fs__WEBPACK_IMPORTED_MODULE_2__.writeFile);

const handleFile = async (sourcePath, {destination, plugins = []}) => {
	if (plugins && !Array.isArray(plugins)) {
		throw new TypeError('The `plugins` option should be an `Array`');
	}

	let data = await readFile(sourcePath);
	data = await (plugins.length > 0 ? (0,p_pipe__WEBPACK_IMPORTED_MODULE_6__.default)(...plugins)(data) : data);

	const {ext} = await file_type__WEBPACK_IMPORTED_MODULE_4__.fromBuffer(data);
	let destinationPath = destination ? path__WEBPACK_IMPORTED_MODULE_1__.join(destination, path__WEBPACK_IMPORTED_MODULE_1__.basename(sourcePath)) : undefined;
	destinationPath = ext === 'webp' ? replace_ext__WEBPACK_IMPORTED_MODULE_7__(destinationPath, '.webp') : destinationPath;

	const returnValue = {
		data,
		sourcePath,
		destinationPath
	};

	if (!destinationPath) {
		return returnValue;
	}

	await fs__WEBPACK_IMPORTED_MODULE_3__.promises.mkdir(path__WEBPACK_IMPORTED_MODULE_1__.dirname(returnValue.destinationPath), {recursive: true});
	await writeFile(returnValue.destinationPath, returnValue.data);

	return returnValue;
};

async function imagemin(input, {glob = true, ...options} = {}) {
	if (!Array.isArray(input)) {
		throw new TypeError(`Expected an \`Array\`, got \`${typeof input}\``);
	}

	const filePaths = glob ? await globby__WEBPACK_IMPORTED_MODULE_5__(input, {onlyFiles: true}) : input;

	return Promise.all(
		filePaths
			.filter(filePath => junk__WEBPACK_IMPORTED_MODULE_8__.not(path__WEBPACK_IMPORTED_MODULE_1__.basename(filePath)))
			.map(async filePath => {
				try {
					return await handleFile(filePath, options);
				} catch (error) {
					error.message = `Error occurred when handling file: ${input}\n\n${error.stack}`;
					throw error;
				}
			})
	);
}

imagemin.buffer = async (input, {plugins = []} = {}) => {
	if (!Buffer.isBuffer(input)) {
		throw new TypeError(`Expected a \`Buffer\`, got \`${typeof input}\``);
	}

	if (plugins.length === 0) {
		return input;
	}

	return (0,p_pipe__WEBPACK_IMPORTED_MODULE_6__.default)(...plugins)(input);
};


/***/ }),

/***/ "./node_modules/imagemin/node_modules/file-type/core.js":
/*!**************************************************************!*\
  !*** ./node_modules/imagemin/node_modules/file-type/core.js ***!
  \**************************************************************/
/***/ ((module, __unused_webpack_exports, __webpack_require__) => {


const Token = __webpack_require__(/*! token-types */ "token-types");
const strtok3 = __webpack_require__(/*! strtok3/lib/core */ "strtok3/lib/core");
const {
	stringToBytes,
	tarHeaderChecksumMatches,
	uint32SyncSafeToken
} = __webpack_require__(/*! ./util */ "./node_modules/imagemin/node_modules/file-type/util.js");
const supported = __webpack_require__(/*! ./supported */ "./node_modules/imagemin/node_modules/file-type/supported.js");

const minimumBytes = 4100; // A fair amount of file-types are detectable within this range

async function fromStream(stream) {
	const tokenizer = await strtok3.fromStream(stream);
	try {
		return await fromTokenizer(tokenizer);
	} finally {
		await tokenizer.close();
	}
}

async function fromBuffer(input) {
	if (!(input instanceof Uint8Array || input instanceof ArrayBuffer || Buffer.isBuffer(input))) {
		throw new TypeError(`Expected the \`input\` argument to be of type \`Uint8Array\` or \`Buffer\` or \`ArrayBuffer\`, got \`${typeof input}\``);
	}

	const buffer = input instanceof Buffer ? input : Buffer.from(input);

	if (!(buffer && buffer.length > 1)) {
		return;
	}

	const tokenizer = strtok3.fromBuffer(buffer);
	return fromTokenizer(tokenizer);
}

function _check(buffer, headers, options) {
	options = {
		offset: 0,
		...options
	};

	for (const [index, header] of headers.entries()) {
		// If a bitmask is set
		if (options.mask) {
			// If header doesn't equal `buf` with bits masked off
			if (header !== (options.mask[index] & buffer[index + options.offset])) {
				return false;
			}
		} else if (header !== buffer[index + options.offset]) {
			return false;
		}
	}

	return true;
}

async function _checkSequence(sequence, tokenizer, ignoreBytes) {
	const buffer = Buffer.alloc(minimumBytes);
	await tokenizer.ignore(ignoreBytes);

	await tokenizer.peekBuffer(buffer, {mayBeLess: true});

	return buffer.includes(Buffer.from(sequence));
}

async function fromTokenizer(tokenizer) {
	try {
		return _fromTokenizer(tokenizer);
	} catch (error) {
		if (!(error instanceof strtok3.EndOfStreamError)) {
			throw error;
		}
	}
}

async function _fromTokenizer(tokenizer) {
	let buffer = Buffer.alloc(minimumBytes);
	const bytesRead = 12;
	const check = (header, options) => _check(buffer, header, options);
	const checkString = (header, options) => check(stringToBytes(header), options);
	const checkSequence = (sequence, ignoreBytes) => _checkSequence(sequence, tokenizer, ignoreBytes);

	// Keep reading until EOF if the file size is unknown.
	if (!tokenizer.fileInfo.size) {
		tokenizer.fileInfo.size = Number.MAX_SAFE_INTEGER;
	}

	await tokenizer.peekBuffer(buffer, {length: bytesRead, mayBeLess: true});

	// -- 2-byte signatures --

	if (check([0x42, 0x4D])) {
		return {
			ext: 'bmp',
			mime: 'image/bmp'
		};
	}

	if (check([0x0B, 0x77])) {
		return {
			ext: 'ac3',
			mime: 'audio/vnd.dolby.dd-raw'
		};
	}

	if (check([0x78, 0x01])) {
		return {
			ext: 'dmg',
			mime: 'application/x-apple-diskimage'
		};
	}

	if (check([0x4D, 0x5A])) {
		return {
			ext: 'exe',
			mime: 'application/x-msdownload'
		};
	}

	if (check([0x25, 0x21])) {
		await tokenizer.peekBuffer(buffer, {length: 24, mayBeLess: true});

		if (checkString('PS-Adobe-', {offset: 2}) &&
			checkString(' EPSF-', {offset: 14})) {
			return {
				ext: 'eps',
				mime: 'application/eps'
			};
		}

		return {
			ext: 'ps',
			mime: 'application/postscript'
		};
	}

	if (
		check([0x1F, 0xA0]) ||
		check([0x1F, 0x9D])
	) {
		return {
			ext: 'Z',
			mime: 'application/x-compress'
		};
	}

	// -- 3-byte signatures --

	if (check([0xFF, 0xD8, 0xFF])) {
		return {
			ext: 'jpg',
			mime: 'image/jpeg'
		};
	}

	if (check([0x49, 0x49, 0xBC])) {
		return {
			ext: 'jxr',
			mime: 'image/vnd.ms-photo'
		};
	}

	if (check([0x1F, 0x8B, 0x8])) {
		return {
			ext: 'gz',
			mime: 'application/gzip'
		};
	}

	if (check([0x42, 0x5A, 0x68])) {
		return {
			ext: 'bz2',
			mime: 'application/x-bzip2'
		};
	}

	if (checkString('ID3')) {
		await tokenizer.ignore(6); // Skip ID3 header until the header size
		const id3HeaderLen = await tokenizer.readToken(uint32SyncSafeToken);
		if (tokenizer.position + id3HeaderLen > tokenizer.fileInfo.size) {
			// Guess file type based on ID3 header for backward compatibility
			return {
				ext: 'mp3',
				mime: 'audio/mpeg'
			};
		}

		await tokenizer.ignore(id3HeaderLen);
		return fromTokenizer(tokenizer); // Skip ID3 header, recursion
	}

	// Musepack, SV7
	if (checkString('MP+')) {
		return {
			ext: 'mpc',
			mime: 'audio/x-musepack'
		};
	}

	if (
		(buffer[0] === 0x43 || buffer[0] === 0x46) &&
		check([0x57, 0x53], {offset: 1})
	) {
		return {
			ext: 'swf',
			mime: 'application/x-shockwave-flash'
		};
	}

	// -- 4-byte signatures --

	if (check([0x47, 0x49, 0x46])) {
		return {
			ext: 'gif',
			mime: 'image/gif'
		};
	}

	if (checkString('FLIF')) {
		return {
			ext: 'flif',
			mime: 'image/flif'
		};
	}

	if (checkString('8BPS')) {
		return {
			ext: 'psd',
			mime: 'image/vnd.adobe.photoshop'
		};
	}

	if (checkString('WEBP', {offset: 8})) {
		return {
			ext: 'webp',
			mime: 'image/webp'
		};
	}

	// Musepack, SV8
	if (checkString('MPCK')) {
		return {
			ext: 'mpc',
			mime: 'audio/x-musepack'
		};
	}

	if (checkString('FORM')) {
		return {
			ext: 'aif',
			mime: 'audio/aiff'
		};
	}

	if (checkString('icns', {offset: 0})) {
		return {
			ext: 'icns',
			mime: 'image/icns'
		};
	}

	// Zip-based file formats
	// Need to be before the `zip` check
	if (check([0x50, 0x4B, 0x3, 0x4])) { // Local file header signature
		try {
			while (tokenizer.position + 30 < tokenizer.fileInfo.size) {
				await tokenizer.readBuffer(buffer, {length: 30});

				// https://en.wikipedia.org/wiki/Zip_(file_format)#File_headers
				const zipHeader = {
					compressedSize: buffer.readUInt32LE(18),
					uncompressedSize: buffer.readUInt32LE(22),
					filenameLength: buffer.readUInt16LE(26),
					extraFieldLength: buffer.readUInt16LE(28)
				};

				zipHeader.filename = await tokenizer.readToken(new Token.StringType(zipHeader.filenameLength, 'utf-8'));
				await tokenizer.ignore(zipHeader.extraFieldLength);

				// Assumes signed `.xpi` from addons.mozilla.org
				if (zipHeader.filename === 'META-INF/mozilla.rsa') {
					return {
						ext: 'xpi',
						mime: 'application/x-xpinstall'
					};
				}

				if (zipHeader.filename.endsWith('.rels') || zipHeader.filename.endsWith('.xml')) {
					const type = zipHeader.filename.split('/')[0];
					switch (type) {
						case '_rels':
							break;
						case 'word':
							return {
								ext: 'docx',
								mime: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
							};
						case 'ppt':
							return {
								ext: 'pptx',
								mime: 'application/vnd.openxmlformats-officedocument.presentationml.presentation'
							};
						case 'xl':
							return {
								ext: 'xlsx',
								mime: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
							};
						default:
							break;
					}
				}

				if (zipHeader.filename.startsWith('xl/')) {
					return {
						ext: 'xlsx',
						mime: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
					};
				}

				// The docx, xlsx and pptx file types extend the Office Open XML file format:
				// https://en.wikipedia.org/wiki/Office_Open_XML_file_formats
				// We look for:
				// - one entry named '[Content_Types].xml' or '_rels/.rels',
				// - one entry indicating specific type of file.
				// MS Office, OpenOffice and LibreOffice may put the parts in different order, so the check should not rely on it.
				if (zipHeader.filename === 'mimetype' && zipHeader.compressedSize === zipHeader.uncompressedSize) {
					const mimeType = await tokenizer.readToken(new Token.StringType(zipHeader.compressedSize, 'utf-8'));

					switch (mimeType) {
						case 'application/epub+zip':
							return {
								ext: 'epub',
								mime: 'application/epub+zip'
							};
						case 'application/vnd.oasis.opendocument.text':
							return {
								ext: 'odt',
								mime: 'application/vnd.oasis.opendocument.text'
							};
						case 'application/vnd.oasis.opendocument.spreadsheet':
							return {
								ext: 'ods',
								mime: 'application/vnd.oasis.opendocument.spreadsheet'
							};
						case 'application/vnd.oasis.opendocument.presentation':
							return {
								ext: 'odp',
								mime: 'application/vnd.oasis.opendocument.presentation'
							};
						default:
					}
				}

				// Try to find next header manually when current one is corrupted
				if (zipHeader.compressedSize === 0) {
					let nextHeaderIndex = -1;

					while (nextHeaderIndex < 0 && (tokenizer.position < tokenizer.fileInfo.size)) {
						await tokenizer.peekBuffer(buffer, {mayBeLess: true});

						nextHeaderIndex = buffer.indexOf('504B0304', 0, 'hex');
						// Move position to the next header if found, skip the whole buffer otherwise
						await tokenizer.ignore(nextHeaderIndex >= 0 ? nextHeaderIndex : buffer.length);
					}
				} else {
					await tokenizer.ignore(zipHeader.compressedSize);
				}
			}
		} catch (error) {
			if (!(error instanceof strtok3.EndOfStreamError)) {
				throw error;
			}
		}

		return {
			ext: 'zip',
			mime: 'application/zip'
		};
	}

	if (checkString('OggS')) {
		// This is an OGG container
		await tokenizer.ignore(28);
		const type = Buffer.alloc(8);
		await tokenizer.readBuffer(type);

		// Needs to be before `ogg` check
		if (_check(type, [0x4F, 0x70, 0x75, 0x73, 0x48, 0x65, 0x61, 0x64])) {
			return {
				ext: 'opus',
				mime: 'audio/opus'
			};
		}

		// If ' theora' in header.
		if (_check(type, [0x80, 0x74, 0x68, 0x65, 0x6F, 0x72, 0x61])) {
			return {
				ext: 'ogv',
				mime: 'video/ogg'
			};
		}

		// If '\x01video' in header.
		if (_check(type, [0x01, 0x76, 0x69, 0x64, 0x65, 0x6F, 0x00])) {
			return {
				ext: 'ogm',
				mime: 'video/ogg'
			};
		}

		// If ' FLAC' in header  https://xiph.org/flac/faq.html
		if (_check(type, [0x7F, 0x46, 0x4C, 0x41, 0x43])) {
			return {
				ext: 'oga',
				mime: 'audio/ogg'
			};
		}

		// 'Speex  ' in header https://en.wikipedia.org/wiki/Speex
		if (_check(type, [0x53, 0x70, 0x65, 0x65, 0x78, 0x20, 0x20])) {
			return {
				ext: 'spx',
				mime: 'audio/ogg'
			};
		}

		// If '\x01vorbis' in header
		if (_check(type, [0x01, 0x76, 0x6F, 0x72, 0x62, 0x69, 0x73])) {
			return {
				ext: 'ogg',
				mime: 'audio/ogg'
			};
		}

		// Default OGG container https://www.iana.org/assignments/media-types/application/ogg
		return {
			ext: 'ogx',
			mime: 'application/ogg'
		};
	}

	if (
		check([0x50, 0x4B]) &&
		(buffer[2] === 0x3 || buffer[2] === 0x5 || buffer[2] === 0x7) &&
		(buffer[3] === 0x4 || buffer[3] === 0x6 || buffer[3] === 0x8)
	) {
		return {
			ext: 'zip',
			mime: 'application/zip'
		};
	}

	//

	// File Type Box (https://en.wikipedia.org/wiki/ISO_base_media_file_format)
	// It's not required to be first, but it's recommended to be. Almost all ISO base media files start with `ftyp` box.
	// `ftyp` box must contain a brand major identifier, which must consist of ISO 8859-1 printable characters.
	// Here we check for 8859-1 printable characters (for simplicity, it's a mask which also catches one non-printable character).
	if (
		checkString('ftyp', {offset: 4}) &&
		(buffer[8] & 0x60) !== 0x00 // Brand major, first character ASCII?
	) {
		// They all can have MIME `video/mp4` except `application/mp4` special-case which is hard to detect.
		// For some cases, we're specific, everything else falls to `video/mp4` with `mp4` extension.
		const brandMajor = buffer.toString('binary', 8, 12).replace('\0', ' ').trim();
		switch (brandMajor) {
			case 'avif':
				return {ext: 'avif', mime: 'image/avif'};
			case 'mif1':
				return {ext: 'heic', mime: 'image/heif'};
			case 'msf1':
				return {ext: 'heic', mime: 'image/heif-sequence'};
			case 'heic':
			case 'heix':
				return {ext: 'heic', mime: 'image/heic'};
			case 'hevc':
			case 'hevx':
				return {ext: 'heic', mime: 'image/heic-sequence'};
			case 'qt':
				return {ext: 'mov', mime: 'video/quicktime'};
			case 'M4V':
			case 'M4VH':
			case 'M4VP':
				return {ext: 'm4v', mime: 'video/x-m4v'};
			case 'M4P':
				return {ext: 'm4p', mime: 'video/mp4'};
			case 'M4B':
				return {ext: 'm4b', mime: 'audio/mp4'};
			case 'M4A':
				return {ext: 'm4a', mime: 'audio/x-m4a'};
			case 'F4V':
				return {ext: 'f4v', mime: 'video/mp4'};
			case 'F4P':
				return {ext: 'f4p', mime: 'video/mp4'};
			case 'F4A':
				return {ext: 'f4a', mime: 'audio/mp4'};
			case 'F4B':
				return {ext: 'f4b', mime: 'audio/mp4'};
			case 'crx':
				return {ext: 'cr3', mime: 'image/x-canon-cr3'};
			default:
				if (brandMajor.startsWith('3g')) {
					if (brandMajor.startsWith('3g2')) {
						return {ext: '3g2', mime: 'video/3gpp2'};
					}

					return {ext: '3gp', mime: 'video/3gpp'};
				}

				return {ext: 'mp4', mime: 'video/mp4'};
		}
	}

	if (checkString('MThd')) {
		return {
			ext: 'mid',
			mime: 'audio/midi'
		};
	}

	if (
		checkString('wOFF') &&
		(
			check([0x00, 0x01, 0x00, 0x00], {offset: 4}) ||
			checkString('OTTO', {offset: 4})
		)
	) {
		return {
			ext: 'woff',
			mime: 'font/woff'
		};
	}

	if (
		checkString('wOF2') &&
		(
			check([0x00, 0x01, 0x00, 0x00], {offset: 4}) ||
			checkString('OTTO', {offset: 4})
		)
	) {
		return {
			ext: 'woff2',
			mime: 'font/woff2'
		};
	}

	if (check([0xD4, 0xC3, 0xB2, 0xA1]) || check([0xA1, 0xB2, 0xC3, 0xD4])) {
		return {
			ext: 'pcap',
			mime: 'application/vnd.tcpdump.pcap'
		};
	}

	// Sony DSD Stream File (DSF)
	if (checkString('DSD ')) {
		return {
			ext: 'dsf',
			mime: 'audio/x-dsf' // Non-standard
		};
	}

	if (checkString('LZIP')) {
		return {
			ext: 'lz',
			mime: 'application/x-lzip'
		};
	}

	if (checkString('fLaC')) {
		return {
			ext: 'flac',
			mime: 'audio/x-flac'
		};
	}

	if (check([0x42, 0x50, 0x47, 0xFB])) {
		return {
			ext: 'bpg',
			mime: 'image/bpg'
		};
	}

	if (checkString('wvpk')) {
		return {
			ext: 'wv',
			mime: 'audio/wavpack'
		};
	}

	if (checkString('%PDF')) {
		// Check if this is an Adobe Illustrator file
		const isAiFile = await checkSequence('Adobe Illustrator', 1350);
		if (isAiFile) {
			return {
				ext: 'ai',
				mime: 'application/postscript'
			};
		}

		// Assume this is just a normal PDF
		return {
			ext: 'pdf',
			mime: 'application/pdf'
		};
	}

	if (check([0x00, 0x61, 0x73, 0x6D])) {
		return {
			ext: 'wasm',
			mime: 'application/wasm'
		};
	}

	// TIFF, little-endian type
	if (check([0x49, 0x49, 0x2A, 0x0])) {
		if (checkString('CR', {offset: 8})) {
			return {
				ext: 'cr2',
				mime: 'image/x-canon-cr2'
			};
		}

		if (check([0x1C, 0x00, 0xFE, 0x00], {offset: 8}) || check([0x1F, 0x00, 0x0B, 0x00], {offset: 8})) {
			return {
				ext: 'nef',
				mime: 'image/x-nikon-nef'
			};
		}

		if (
			check([0x08, 0x00, 0x00, 0x00], {offset: 4}) &&
			(check([0x2D, 0x00, 0xFE, 0x00], {offset: 8}) ||
				check([0x27, 0x00, 0xFE, 0x00], {offset: 8}))
		) {
			return {
				ext: 'dng',
				mime: 'image/x-adobe-dng'
			};
		}

		buffer = Buffer.alloc(24);
		await tokenizer.peekBuffer(buffer);
		if (
			(check([0x10, 0xFB, 0x86, 0x01], {offset: 4}) || check([0x08, 0x00, 0x00, 0x00], {offset: 4})) &&
			// This pattern differentiates ARW from other TIFF-ish file types:
			check([0x00, 0xFE, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x01], {offset: 9})
		) {
			return {
				ext: 'arw',
				mime: 'image/x-sony-arw'
			};
		}

		return {
			ext: 'tif',
			mime: 'image/tiff'
		};
	}

	// TIFF, big-endian type
	if (check([0x4D, 0x4D, 0x0, 0x2A])) {
		return {
			ext: 'tif',
			mime: 'image/tiff'
		};
	}

	if (checkString('MAC ')) {
		return {
			ext: 'ape',
			mime: 'audio/ape'
		};
	}

	// https://github.com/threatstack/libmagic/blob/master/magic/Magdir/matroska
	if (check([0x1A, 0x45, 0xDF, 0xA3])) { // Root element: EBML
		async function readField() {
			const msb = await tokenizer.peekNumber(Token.UINT8);
			let mask = 0x80;
			let ic = 0; // 0 = A, 1 = B, 2 = C, 3 = D

			while ((msb & mask) === 0) {
				++ic;
				mask >>= 1;
			}

			const id = Buffer.alloc(ic + 1);
			await tokenizer.readBuffer(id);
			return id;
		}

		async function readElement() {
			const id = await readField();
			const lenField = await readField();
			lenField[0] ^= 0x80 >> (lenField.length - 1);
			const nrLen = Math.min(6, lenField.length); // JavaScript can max read 6 bytes integer
			return {
				id: id.readUIntBE(0, id.length),
				len: lenField.readUIntBE(lenField.length - nrLen, nrLen)
			};
		}

		async function readChildren(level, children) {
			while (children > 0) {
				const e = await readElement();
				if (e.id === 0x4282) {
					return tokenizer.readToken(new Token.StringType(e.len, 'utf-8')); // Return DocType
				}

				await tokenizer.ignore(e.len); // ignore payload
				--children;
			}
		}

		const re = await readElement();
		const docType = await readChildren(1, re.len);

		switch (docType) {
			case 'webm':
				return {
					ext: 'webm',
					mime: 'video/webm'
				};

			case 'matroska':
				return {
					ext: 'mkv',
					mime: 'video/x-matroska'
				};

			default:
				return;
		}
	}

	// RIFF file format which might be AVI, WAV, QCP, etc
	if (check([0x52, 0x49, 0x46, 0x46])) {
		if (check([0x41, 0x56, 0x49], {offset: 8})) {
			return {
				ext: 'avi',
				mime: 'video/vnd.avi'
			};
		}

		if (check([0x57, 0x41, 0x56, 0x45], {offset: 8})) {
			return {
				ext: 'wav',
				mime: 'audio/vnd.wave'
			};
		}

		// QLCM, QCP file
		if (check([0x51, 0x4C, 0x43, 0x4D], {offset: 8})) {
			return {
				ext: 'qcp',
				mime: 'audio/qcelp'
			};
		}
	}

	if (checkString('SQLi')) {
		return {
			ext: 'sqlite',
			mime: 'application/x-sqlite3'
		};
	}

	if (check([0x4E, 0x45, 0x53, 0x1A])) {
		return {
			ext: 'nes',
			mime: 'application/x-nintendo-nes-rom'
		};
	}

	if (checkString('Cr24')) {
		return {
			ext: 'crx',
			mime: 'application/x-google-chrome-extension'
		};
	}

	if (
		checkString('MSCF') ||
		checkString('ISc(')
	) {
		return {
			ext: 'cab',
			mime: 'application/vnd.ms-cab-compressed'
		};
	}

	if (check([0xED, 0xAB, 0xEE, 0xDB])) {
		return {
			ext: 'rpm',
			mime: 'application/x-rpm'
		};
	}

	if (check([0xC5, 0xD0, 0xD3, 0xC6])) {
		return {
			ext: 'eps',
			mime: 'application/eps'
		};
	}

	// -- 5-byte signatures --

	if (check([0x4F, 0x54, 0x54, 0x4F, 0x00])) {
		return {
			ext: 'otf',
			mime: 'font/otf'
		};
	}

	if (checkString('#!AMR')) {
		return {
			ext: 'amr',
			mime: 'audio/amr'
		};
	}

	if (checkString('{\\rtf')) {
		return {
			ext: 'rtf',
			mime: 'application/rtf'
		};
	}

	if (check([0x46, 0x4C, 0x56, 0x01])) {
		return {
			ext: 'flv',
			mime: 'video/x-flv'
		};
	}

	if (checkString('IMPM')) {
		return {
			ext: 'it',
			mime: 'audio/x-it'
		};
	}

	if (
		checkString('-lh0-', {offset: 2}) ||
		checkString('-lh1-', {offset: 2}) ||
		checkString('-lh2-', {offset: 2}) ||
		checkString('-lh3-', {offset: 2}) ||
		checkString('-lh4-', {offset: 2}) ||
		checkString('-lh5-', {offset: 2}) ||
		checkString('-lh6-', {offset: 2}) ||
		checkString('-lh7-', {offset: 2}) ||
		checkString('-lzs-', {offset: 2}) ||
		checkString('-lz4-', {offset: 2}) ||
		checkString('-lz5-', {offset: 2}) ||
		checkString('-lhd-', {offset: 2})
	) {
		return {
			ext: 'lzh',
			mime: 'application/x-lzh-compressed'
		};
	}

	// MPEG program stream (PS or MPEG-PS)
	if (check([0x00, 0x00, 0x01, 0xBA])) {
		//  MPEG-PS, MPEG-1 Part 1
		if (check([0x21], {offset: 4, mask: [0xF1]})) {
			return {
				ext: 'mpg', // May also be .ps, .mpeg
				mime: 'video/MP1S'
			};
		}

		// MPEG-PS, MPEG-2 Part 1
		if (check([0x44], {offset: 4, mask: [0xC4]})) {
			return {
				ext: 'mpg', // May also be .mpg, .m2p, .vob or .sub
				mime: 'video/MP2P'
			};
		}
	}

	// -- 6-byte signatures --

	if (check([0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00])) {
		return {
			ext: 'xz',
			mime: 'application/x-xz'
		};
	}

	if (checkString('<?xml ')) {
		return {
			ext: 'xml',
			mime: 'application/xml'
		};
	}

	if (checkString('BEGIN:')) {
		return {
			ext: 'ics',
			mime: 'text/calendar'
		};
	}

	if (check([0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C])) {
		return {
			ext: '7z',
			mime: 'application/x-7z-compressed'
		};
	}

	if (
		check([0x52, 0x61, 0x72, 0x21, 0x1A, 0x7]) &&
		(buffer[6] === 0x0 || buffer[6] === 0x1)
	) {
		return {
			ext: 'rar',
			mime: 'application/x-rar-compressed'
		};
	}

	// -- 7-byte signatures --

	if (checkString('BLENDER')) {
		return {
			ext: 'blend',
			mime: 'application/x-blender'
		};
	}

	if (checkString('!<arch>')) {
		await tokenizer.ignore(8);
		const str = await tokenizer.readToken(new Token.StringType(13, 'ascii'));
		if (str === 'debian-binary') {
			return {
				ext: 'deb',
				mime: 'application/x-deb'
			};
		}

		return {
			ext: 'ar',
			mime: 'application/x-unix-archive'
		};
	}

	// -- 8-byte signatures --

	if (check([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])) {
		// APNG format (https://wiki.mozilla.org/APNG_Specification)
		// 1. Find the first IDAT (image data) chunk (49 44 41 54)
		// 2. Check if there is an "acTL" chunk before the IDAT one (61 63 54 4C)

		// Offset calculated as follows:
		// - 8 bytes: PNG signature
		// - 4 (length) + 4 (chunk type) + 13 (chunk data) + 4 (CRC): IHDR chunk

		await tokenizer.ignore(8); // ignore PNG signature

		async function readChunkHeader() {
			return {
				length: await tokenizer.readToken(Token.INT32_BE),
				type: await tokenizer.readToken(new Token.StringType(4, 'binary'))
			};
		}

		do {
			const chunk = await readChunkHeader();
			switch (chunk.type) {
				case 'IDAT':
					return {
						ext: 'png',
						mime: 'image/png'
					};
				case 'acTL':
					return {
						ext: 'apng',
						mime: 'image/apng'
					};
				default:
					await tokenizer.ignore(chunk.length + 4); // Ignore chunk-data + CRC
			}
		} while (tokenizer.position < tokenizer.fileInfo.size);

		return {
			ext: 'png',
			mime: 'image/png'
		};
	}

	if (check([0x41, 0x52, 0x52, 0x4F, 0x57, 0x31, 0x00, 0x00])) {
		return {
			ext: 'arrow',
			mime: 'application/x-apache-arrow'
		};
	}

	if (check([0x67, 0x6C, 0x54, 0x46, 0x02, 0x00, 0x00, 0x00])) {
		return {
			ext: 'glb',
			mime: 'model/gltf-binary'
		};
	}

	// `mov` format variants
	if (
		check([0x66, 0x72, 0x65, 0x65], {offset: 4}) || // `free`
		check([0x6D, 0x64, 0x61, 0x74], {offset: 4}) || // `mdat` MJPEG
		check([0x6D, 0x6F, 0x6F, 0x76], {offset: 4}) || // `moov`
		check([0x77, 0x69, 0x64, 0x65], {offset: 4}) // `wide`
	) {
		return {
			ext: 'mov',
			mime: 'video/quicktime'
		};
	}

	// -- 9-byte signatures --

	if (check([0x49, 0x49, 0x52, 0x4F, 0x08, 0x00, 0x00, 0x00, 0x18])) {
		return {
			ext: 'orf',
			mime: 'image/x-olympus-orf'
		};
	}

	// -- 12-byte signatures --

	if (check([0x49, 0x49, 0x55, 0x00, 0x18, 0x00, 0x00, 0x00, 0x88, 0xE7, 0x74, 0xD8])) {
		return {
			ext: 'rw2',
			mime: 'image/x-panasonic-rw2'
		};
	}

	// ASF_Header_Object first 80 bytes
	if (check([0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9])) {
		async function readHeader() {
			const guid = Buffer.alloc(16);
			await tokenizer.readBuffer(guid);
			return {
				id: guid,
				size: await tokenizer.readToken(Token.UINT64_LE)
			};
		}

		await tokenizer.ignore(30);
		// Search for header should be in first 1KB of file.
		while (tokenizer.position + 24 < tokenizer.fileInfo.size) {
			const header = await readHeader();
			let payload = header.size - 24;
			if (_check(header.id, [0x91, 0x07, 0xDC, 0xB7, 0xB7, 0xA9, 0xCF, 0x11, 0x8E, 0xE6, 0x00, 0xC0, 0x0C, 0x20, 0x53, 0x65])) {
				// Sync on Stream-Properties-Object (B7DC0791-A9B7-11CF-8EE6-00C00C205365)
				const typeId = Buffer.alloc(16);
				payload -= await tokenizer.readBuffer(typeId);

				if (_check(typeId, [0x40, 0x9E, 0x69, 0xF8, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B])) {
					// Found audio:
					return {
						ext: 'wma',
						mime: 'audio/x-ms-wma'
					};
				}

				if (_check(typeId, [0xC0, 0xEF, 0x19, 0xBC, 0x4D, 0x5B, 0xCF, 0x11, 0xA8, 0xFD, 0x00, 0x80, 0x5F, 0x5C, 0x44, 0x2B])) {
					// Found video:
					return {
						ext: 'wmv',
						mime: 'video/x-ms-asf'
					};
				}

				break;
			}

			await tokenizer.ignore(payload);
		}

		// Default to ASF generic extension
		return {
			ext: 'asf',
			mime: 'application/vnd.ms-asf'
		};
	}

	if (check([0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A])) {
		return {
			ext: 'ktx',
			mime: 'image/ktx'
		};
	}

	if ((check([0x7E, 0x10, 0x04]) || check([0x7E, 0x18, 0x04])) && check([0x30, 0x4D, 0x49, 0x45], {offset: 4})) {
		return {
			ext: 'mie',
			mime: 'application/x-mie'
		};
	}

	if (check([0x27, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], {offset: 2})) {
		return {
			ext: 'shp',
			mime: 'application/x-esri-shape'
		};
	}

	if (check([0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20, 0x0D, 0x0A, 0x87, 0x0A])) {
		// JPEG-2000 family

		await tokenizer.ignore(20);
		const type = await tokenizer.readToken(new Token.StringType(4, 'ascii'));
		switch (type) {
			case 'jp2 ':
				return {
					ext: 'jp2',
					mime: 'image/jp2'
				};
			case 'jpx ':
				return {
					ext: 'jpx',
					mime: 'image/jpx'
				};
			case 'jpm ':
				return {
					ext: 'jpm',
					mime: 'image/jpm'
				};
			case 'mjp2':
				return {
					ext: 'mj2',
					mime: 'image/mj2'
				};
			default:
				return;
		}
	}

	// -- Unsafe signatures --

	if (
		check([0x0, 0x0, 0x1, 0xBA]) ||
		check([0x0, 0x0, 0x1, 0xB3])
	) {
		return {
			ext: 'mpg',
			mime: 'video/mpeg'
		};
	}

	if (check([0x00, 0x01, 0x00, 0x00, 0x00])) {
		return {
			ext: 'ttf',
			mime: 'font/ttf'
		};
	}

	if (check([0x00, 0x00, 0x01, 0x00])) {
		return {
			ext: 'ico',
			mime: 'image/x-icon'
		};
	}

	if (check([0x00, 0x00, 0x02, 0x00])) {
		return {
			ext: 'cur',
			mime: 'image/x-icon'
		};
	}

	// Increase sample size from 12 to 256.
	await tokenizer.peekBuffer(buffer, {length: Math.min(256, tokenizer.fileInfo.size), mayBeLess: true});

	// `raf` is here just to keep all the raw image detectors together.
	if (checkString('FUJIFILMCCD-RAW')) {
		return {
			ext: 'raf',
			mime: 'image/x-fujifilm-raf'
		};
	}

	if (checkString('Extended Module:')) {
		return {
			ext: 'xm',
			mime: 'audio/x-xm'
		};
	}

	if (checkString('Creative Voice File')) {
		return {
			ext: 'voc',
			mime: 'audio/x-voc'
		};
	}

	if (check([0x04, 0x00, 0x00, 0x00]) && buffer.length >= 16) { // Rough & quick check Pickle/ASAR
		const jsonSize = buffer.readUInt32LE(12);
		if (jsonSize > 12 && jsonSize < 240 && buffer.length >= jsonSize + 16) {
			try {
				const header = buffer.slice(16, jsonSize + 16).toString();
				const json = JSON.parse(header);
				// Check if Pickle is ASAR
				if (json.files) { // Final check, assuring Pickle/ASAR format
					return {
						ext: 'asar',
						mime: 'application/x-asar'
					};
				}
			} catch (_) {
			}
		}
	}

	if (check([0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3E])) {
		return {
			ext: 'msi',
			mime: 'application/x-msi'
		};
	}

	if (check([0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x02])) {
		return {
			ext: 'mxf',
			mime: 'application/mxf'
		};
	}

	if (checkString('SCRM', {offset: 44})) {
		return {
			ext: 's3m',
			mime: 'audio/x-s3m'
		};
	}

	if (check([0x47], {offset: 4}) && (check([0x47], {offset: 192}) || check([0x47], {offset: 196}))) {
		return {
			ext: 'mts',
			mime: 'video/mp2t'
		};
	}

	if (check([0x42, 0x4F, 0x4F, 0x4B, 0x4D, 0x4F, 0x42, 0x49], {offset: 60})) {
		return {
			ext: 'mobi',
			mime: 'application/x-mobipocket-ebook'
		};
	}

	if (check([0x44, 0x49, 0x43, 0x4D], {offset: 128})) {
		return {
			ext: 'dcm',
			mime: 'application/dicom'
		};
	}

	if (check([0x4C, 0x00, 0x00, 0x00, 0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46])) {
		return {
			ext: 'lnk',
			mime: 'application/x.ms.shortcut' // Invented by us
		};
	}

	if (check([0x62, 0x6F, 0x6F, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x6D, 0x61, 0x72, 0x6B, 0x00, 0x00, 0x00, 0x00])) {
		return {
			ext: 'alias',
			mime: 'application/x.apple.alias' // Invented by us
		};
	}

	if (
		check([0x4C, 0x50], {offset: 34}) &&
		(
			check([0x00, 0x00, 0x01], {offset: 8}) ||
			check([0x01, 0x00, 0x02], {offset: 8}) ||
			check([0x02, 0x00, 0x02], {offset: 8})
		)
	) {
		return {
			ext: 'eot',
			mime: 'application/vnd.ms-fontobject'
		};
	}

	if (check([0x06, 0x06, 0xED, 0xF5, 0xD8, 0x1D, 0x46, 0xE5, 0xBD, 0x31, 0xEF, 0xE7, 0xFE, 0x74, 0xB7, 0x1D])) {
		return {
			ext: 'indd',
			mime: 'application/x-indesign'
		};
	}

	// Increase sample size from 256 to 512
	await tokenizer.peekBuffer(buffer, {length: Math.min(512, tokenizer.fileInfo.size), mayBeLess: true});

	// Requires a buffer size of 512 bytes
	if (tarHeaderChecksumMatches(buffer)) {
		return {
			ext: 'tar',
			mime: 'application/x-tar'
		};
	}

	if (check([0xFF, 0xFE, 0xFF, 0x0E, 0x53, 0x00, 0x6B, 0x00, 0x65, 0x00, 0x74, 0x00, 0x63, 0x00, 0x68, 0x00, 0x55, 0x00, 0x70, 0x00, 0x20, 0x00, 0x4D, 0x00, 0x6F, 0x00, 0x64, 0x00, 0x65, 0x00, 0x6C, 0x00])) {
		return {
			ext: 'skp',
			mime: 'application/vnd.sketchup.skp'
		};
	}

	if (checkString('-----BEGIN PGP MESSAGE-----')) {
		return {
			ext: 'pgp',
			mime: 'application/pgp-encrypted'
		};
	}

	// Check for MPEG header at different starting offsets
	for (let start = 0; start < 2 && start < (buffer.length - 16); start++) {
		// Check MPEG 1 or 2 Layer 3 header, or 'layer 0' for ADTS (MPEG sync-word 0xFFE)
		if (buffer.length >= start + 2 && check([0xFF, 0xE0], {offset: start, mask: [0xFF, 0xE0]})) {
			if (check([0x10], {offset: start + 1, mask: [0x16]})) {
				// Check for (ADTS) MPEG-2
				if (check([0x08], {offset: start + 1, mask: [0x08]})) {
					return {
						ext: 'aac',
						mime: 'audio/aac'
					};
				}

				// Must be (ADTS) MPEG-4
				return {
					ext: 'aac',
					mime: 'audio/aac'
				};
			}

			// MPEG 1 or 2 Layer 3 header
			// Check for MPEG layer 3
			if (check([0x02], {offset: start + 1, mask: [0x06]})) {
				return {
					ext: 'mp3',
					mime: 'audio/mpeg'
				};
			}

			// Check for MPEG layer 2
			if (check([0x04], {offset: start + 1, mask: [0x06]})) {
				return {
					ext: 'mp2',
					mime: 'audio/mpeg'
				};
			}

			// Check for MPEG layer 1
			if (check([0x06], {offset: start + 1, mask: [0x06]})) {
				return {
					ext: 'mp1',
					mime: 'audio/mpeg'
				};
			}
		}
	}
}

const stream = readableStream => new Promise((resolve, reject) => {
	// Using `eval` to work around issues when bundling with Webpack
	const stream = eval('require')('stream'); // eslint-disable-line no-eval

	readableStream.on('error', reject);
	readableStream.once('readable', async () => {
		// Set up output stream
		const pass = new stream.PassThrough();
		let outputStream;
		if (stream.pipeline) {
			outputStream = stream.pipeline(readableStream, pass, () => {
			});
		} else {
			outputStream = readableStream.pipe(pass);
		}

		// Read the input stream and detect the filetype
		const chunk = readableStream.read(minimumBytes) || readableStream.read() || Buffer.alloc(0);
		try {
			const fileType = await fromBuffer(chunk);
			pass.fileType = fileType;
		} catch (error) {
			reject(error);
		}

		resolve(outputStream);
	});
});

const fileType = {
	fromStream,
	fromTokenizer,
	fromBuffer,
	stream
};

Object.defineProperty(fileType, 'extensions', {
	get() {
		return new Set(supported.extensions);
	}
});

Object.defineProperty(fileType, 'mimeTypes', {
	get() {
		return new Set(supported.mimeTypes);
	}
});

module.exports = fileType;


/***/ }),

/***/ "./node_modules/imagemin/node_modules/file-type/index.js":
/*!***************************************************************!*\
  !*** ./node_modules/imagemin/node_modules/file-type/index.js ***!
  \***************************************************************/
/***/ ((module, __unused_webpack_exports, __webpack_require__) => {


const strtok3 = __webpack_require__(/*! strtok3 */ "strtok3");
const core = __webpack_require__(/*! ./core */ "./node_modules/imagemin/node_modules/file-type/core.js");

async function fromFile(path) {
	const tokenizer = await strtok3.fromFile(path);
	try {
		return await core.fromTokenizer(tokenizer);
	} finally {
		await tokenizer.close();
	}
}

const fileType = {
	fromFile
};

Object.assign(fileType, core);

Object.defineProperty(fileType, 'extensions', {
	get() {
		return core.extensions;
	}
});

Object.defineProperty(fileType, 'mimeTypes', {
	get() {
		return core.mimeTypes;
	}
});

module.exports = fileType;


/***/ }),

/***/ "./node_modules/imagemin/node_modules/file-type/supported.js":
/*!*******************************************************************!*\
  !*** ./node_modules/imagemin/node_modules/file-type/supported.js ***!
  \*******************************************************************/
/***/ ((module) => {



module.exports = {
	extensions: [
		'jpg',
		'png',
		'apng',
		'gif',
		'webp',
		'flif',
		'cr2',
		'cr3',
		'orf',
		'arw',
		'dng',
		'nef',
		'rw2',
		'raf',
		'tif',
		'bmp',
		'icns',
		'jxr',
		'psd',
		'indd',
		'zip',
		'tar',
		'rar',
		'gz',
		'bz2',
		'7z',
		'dmg',
		'mp4',
		'mid',
		'mkv',
		'webm',
		'mov',
		'avi',
		'mpg',
		'mp2',
		'mp3',
		'm4a',
		'oga',
		'ogg',
		'ogv',
		'opus',
		'flac',
		'wav',
		'spx',
		'amr',
		'pdf',
		'epub',
		'exe',
		'swf',
		'rtf',
		'wasm',
		'woff',
		'woff2',
		'eot',
		'ttf',
		'otf',
		'ico',
		'flv',
		'ps',
		'xz',
		'sqlite',
		'nes',
		'crx',
		'xpi',
		'cab',
		'deb',
		'ar',
		'rpm',
		'Z',
		'lz',
		'msi',
		'mxf',
		'mts',
		'blend',
		'bpg',
		'docx',
		'pptx',
		'xlsx',
		'3gp',
		'3g2',
		'jp2',
		'jpm',
		'jpx',
		'mj2',
		'aif',
		'qcp',
		'odt',
		'ods',
		'odp',
		'xml',
		'mobi',
		'heic',
		'cur',
		'ktx',
		'ape',
		'wv',
		'wmv',
		'wma',
		'dcm',
		'ics',
		'glb',
		'pcap',
		'dsf',
		'lnk',
		'alias',
		'voc',
		'ac3',
		'm4v',
		'm4p',
		'm4b',
		'f4v',
		'f4p',
		'f4b',
		'f4a',
		'mie',
		'asf',
		'ogm',
		'ogx',
		'mpc',
		'arrow',
		'shp',
		'aac',
		'mp1',
		'it',
		's3m',
		'xm',
		'ai',
		'skp',
		'avif',
		'eps',
		'lzh',
		'pgp',
		'asar'
	],
	mimeTypes: [
		'image/jpeg',
		'image/png',
		'image/gif',
		'image/webp',
		'image/flif',
		'image/x-canon-cr2',
		'image/x-canon-cr3',
		'image/tiff',
		'image/bmp',
		'image/vnd.ms-photo',
		'image/vnd.adobe.photoshop',
		'application/x-indesign',
		'application/epub+zip',
		'application/x-xpinstall',
		'application/vnd.oasis.opendocument.text',
		'application/vnd.oasis.opendocument.spreadsheet',
		'application/vnd.oasis.opendocument.presentation',
		'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
		'application/vnd.openxmlformats-officedocument.presentationml.presentation',
		'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
		'application/zip',
		'application/x-tar',
		'application/x-rar-compressed',
		'application/gzip',
		'application/x-bzip2',
		'application/x-7z-compressed',
		'application/x-apple-diskimage',
		'application/x-apache-arrow',
		'video/mp4',
		'audio/midi',
		'video/x-matroska',
		'video/webm',
		'video/quicktime',
		'video/vnd.avi',
		'audio/vnd.wave',
		'audio/qcelp',
		'audio/x-ms-wma',
		'video/x-ms-asf',
		'application/vnd.ms-asf',
		'video/mpeg',
		'video/3gpp',
		'audio/mpeg',
		'audio/mp4', // RFC 4337
		'audio/opus',
		'video/ogg',
		'audio/ogg',
		'application/ogg',
		'audio/x-flac',
		'audio/ape',
		'audio/wavpack',
		'audio/amr',
		'application/pdf',
		'application/x-msdownload',
		'application/x-shockwave-flash',
		'application/rtf',
		'application/wasm',
		'font/woff',
		'font/woff2',
		'application/vnd.ms-fontobject',
		'font/ttf',
		'font/otf',
		'image/x-icon',
		'video/x-flv',
		'application/postscript',
		'application/eps',
		'application/x-xz',
		'application/x-sqlite3',
		'application/x-nintendo-nes-rom',
		'application/x-google-chrome-extension',
		'application/vnd.ms-cab-compressed',
		'application/x-deb',
		'application/x-unix-archive',
		'application/x-rpm',
		'application/x-compress',
		'application/x-lzip',
		'application/x-msi',
		'application/x-mie',
		'application/mxf',
		'video/mp2t',
		'application/x-blender',
		'image/bpg',
		'image/jp2',
		'image/jpx',
		'image/jpm',
		'image/mj2',
		'audio/aiff',
		'application/xml',
		'application/x-mobipocket-ebook',
		'image/heif',
		'image/heif-sequence',
		'image/heic',
		'image/heic-sequence',
		'image/icns',
		'image/ktx',
		'application/dicom',
		'audio/x-musepack',
		'text/calendar',
		'model/gltf-binary',
		'application/vnd.tcpdump.pcap',
		'audio/x-dsf', // Non-standard
		'application/x.ms.shortcut', // Invented by us
		'application/x.apple.alias', // Invented by us
		'audio/x-voc',
		'audio/vnd.dolby.dd-raw',
		'audio/x-m4a',
		'image/apng',
		'image/x-olympus-orf',
		'image/x-sony-arw',
		'image/x-adobe-dng',
		'image/x-nikon-nef',
		'image/x-panasonic-rw2',
		'image/x-fujifilm-raf',
		'video/x-m4v',
		'video/3gpp2',
		'application/x-esri-shape',
		'audio/aac',
		'audio/x-it',
		'audio/x-s3m',
		'audio/x-xm',
		'video/MP1S',
		'video/MP2P',
		'application/vnd.sketchup.skp',
		'image/avif',
		'application/x-lzh-compressed',
		'application/pgp-encrypted',
		'application/x-asar'
	]
};


/***/ }),

/***/ "./node_modules/imagemin/node_modules/file-type/util.js":
/*!**************************************************************!*\
  !*** ./node_modules/imagemin/node_modules/file-type/util.js ***!
  \**************************************************************/
/***/ ((__unused_webpack_module, exports) => {



exports.stringToBytes = string => [...string].map(character => character.charCodeAt(0));

exports.tarHeaderChecksumMatches = buffer => { // Does not check if checksum field characters are valid
	if (buffer.length < 512) { // `tar` header size, cannot compute checksum without it
		return false;
	}

	const readSum = parseInt(buffer.toString('utf8', 148, 154).replace(/\0.*$/, '').trim(), 8); // Read sum in header
	if (isNaN(readSum)) {
		return false;
	}

	const MASK_8TH_BIT = 0x80;

	let sum = 256; // Initialize sum, with 256 as sum of 8 spaces in checksum field
	let signedBitSum = 0; // Initialize signed bit sum

	for (let i = 0; i < 148; i++) {
		const byte = buffer[i];
		sum += byte;
		signedBitSum += byte & MASK_8TH_BIT; // Add signed bit to signed bit sum
	}

	// Skip checksum field

	for (let i = 156; i < 512; i++) {
		const byte = buffer[i];
		sum += byte;
		signedBitSum += byte & MASK_8TH_BIT; // Add signed bit to signed bit sum
	}

	// Some implementations compute checksum incorrectly using signed bytes
	return (
		// Checksum in header equals the sum we calculated
		readSum === sum ||

		// Checksum in header equals sum we calculated plus signed-to-unsigned delta
		readSum === (sum - (signedBitSum << 1))
	);
};

/**
ID3 UINT32 sync-safe tokenizer token.
28 bits (representing up to 256MB) integer, the msb is 0 to avoid "false syncsignals".
*/
exports.uint32SyncSafeToken = {
	get: (buffer, offset) => {
		return (buffer[offset + 3] & 0x7F) | ((buffer[offset + 2]) << 7) | ((buffer[offset + 1]) << 14) | ((buffer[offset]) << 21);
	},
	len: 4
};


/***/ }),

/***/ "./node_modules/is-png/index.js":
/*!**************************************!*\
  !*** ./node_modules/is-png/index.js ***!
  \**************************************/
/***/ ((__unused_webpack___webpack_module__, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "default": () => (/* binding */ isPng)
/* harmony export */ });
function isPng(buffer) {
	if (!buffer || buffer.length < 8) {
		return false;
	}

	return (
		buffer[0] === 0x89 &&
		buffer[1] === 0x50 &&
		buffer[2] === 0x4E &&
		buffer[3] === 0x47 &&
		buffer[4] === 0x0D &&
		buffer[5] === 0x0A &&
		buffer[6] === 0x1A &&
		buffer[7] === 0x0A
	);
}


/***/ }),

/***/ "./node_modules/p-pipe/index.js":
/*!**************************************!*\
  !*** ./node_modules/p-pipe/index.js ***!
  \**************************************/
/***/ ((__unused_webpack___webpack_module__, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "default": () => (/* binding */ pPipe)
/* harmony export */ });
function pPipe(...functions) {
	if (functions.length === 0) {
		throw new Error('Expected at least one argument');
	}

	return async input => {
		let currentValue = input;

		for (const function_ of functions) {
			currentValue = await function_(currentValue); // eslint-disable-line no-await-in-loop
		}

		return currentValue;
	};
}


/***/ }),

/***/ "./src/api/tauricon.ts":
/*!*****************************!*\
  !*** ./src/api/tauricon.ts ***!
  \*****************************/
/***/ (function(module, exports, __webpack_require__) {

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
/* eslint-disable @typescript-eslint/restrict-template-expressions, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call */
/**
 * This is a module that takes an original image and resizes
 * it to common icon sizes and will put them in a folder.
 * It will retain transparency and can make special file
 * types. You can control the settings.
 *
 * @module tauricon
 * @exports tauricon
 * @author Daniel Thompson-Yvetot
 * @license MIT
 */
var fs_extra_1 = __webpack_require__(/*! fs-extra */ "fs-extra");
var imagemin_1 = __importDefault(__webpack_require__(/*! imagemin */ "./node_modules/imagemin/index.js"));
var imagemin_optipng_1 = __importDefault(__webpack_require__(/*! imagemin-optipng */ "imagemin-optipng"));
var imagemin_zopfli_1 = __importDefault(__webpack_require__(/*! imagemin-zopfli */ "imagemin-zopfli"));
var is_png_1 = __importDefault(__webpack_require__(/*! is-png */ "./node_modules/is-png/index.js"));
var path_1 = __importDefault(__webpack_require__(/*! path */ "path"));
var png2icons = __importStar(__webpack_require__(/*! png2icons */ "png2icons"));
var read_chunk_1 = __importDefault(__webpack_require__(/*! read-chunk */ "read-chunk"));
var sharp_1 = __importDefault(__webpack_require__(/*! sharp */ "sharp"));
var app_paths_1 = __webpack_require__(/*! ../helpers/app-paths */ "./src/helpers/app-paths.ts");
var logger_1 = __importDefault(__webpack_require__(/*! ../helpers/logger */ "./src/helpers/logger.ts"));
var settings = __importStar(__webpack_require__(/*! ../helpers/tauricon.config */ "./src/helpers/tauricon.config.ts"));
var chalk_1 = __importDefault(__webpack_require__(/*! chalk */ "chalk"));
var package_json_1 = __webpack_require__(/*! ../../package.json */ "./package.json");
var log = logger_1.default('app:spawn');
var warn = logger_1.default('app:spawn', chalk_1.default.red);
var image = false;
var spinnerInterval = null;
var exists = function (file) {
    return __awaiter(this, void 0, void 0, function () {
        var err_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4 /*yield*/, fs_extra_1.access(file)];
                case 1:
                    _a.sent();
                    return [2 /*return*/, true];
                case 2:
                    err_1 = _a.sent();
                    return [2 /*return*/, false];
                case 3: return [2 /*return*/];
            }
        });
    });
};
/**
 * This is the first call that attempts to memoize the sharp(src).
 * If the source image cannot be found or if it is not a png, it
 * is a failsafe that will exit or throw.
 *
 * @param {string} src - a folder to target
 * @exits {error} if not a png, if not an image
 */
var checkSrc = function (src) { return __awaiter(void 0, void 0, void 0, function () {
    var srcExists, buffer, meta, stats;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0:
                if (!(image !== false)) return [3 /*break*/, 1];
                return [2 /*return*/, image];
            case 1: return [4 /*yield*/, exists(src)];
            case 2:
                srcExists = _a.sent();
                if (!!srcExists) return [3 /*break*/, 3];
                image = false;
                if (spinnerInterval)
                    clearInterval(spinnerInterval);
                warn('[ERROR] Source image for tauricon not found');
                process.exit(1);
                return [3 /*break*/, 8];
            case 3: return [4 /*yield*/, read_chunk_1.default(src, 0, 8)];
            case 4:
                buffer = _a.sent();
                if (!is_png_1.default(buffer)) return [3 /*break*/, 7];
                image = sharp_1.default(src);
                return [4 /*yield*/, image.metadata()];
            case 5:
                meta = _a.sent();
                if (!meta.hasAlpha || meta.channels !== 4) {
                    if (spinnerInterval)
                        clearInterval(spinnerInterval);
                    warn('[ERROR] Source png for tauricon is not transparent');
                    process.exit(1);
                }
                return [4 /*yield*/, image.stats()];
            case 6:
                stats = _a.sent();
                if (stats.isOpaque) {
                    if (spinnerInterval)
                        clearInterval(spinnerInterval);
                    warn('[ERROR] Source png for tauricon could not be detected as transparent');
                    process.exit(1);
                }
                return [2 /*return*/, image];
            case 7:
                image = false;
                if (spinnerInterval)
                    clearInterval(spinnerInterval);
                warn('[ERROR] Source image for tauricon is not a png');
                process.exit(1);
                _a.label = 8;
            case 8: return [2 /*return*/];
        }
    });
}); };
/**
 * Sort the folders in the current job for unique folders.
 *
 * @param {object} options - a subset of the settings
 * @returns {array} folders
 */
// TODO: proper type of options and folders
var uniqueFolders = function (options) {
    var folders = [];
    for (var type in options) {
        var option = options[String(type)];
        if (option.folder) {
            folders.push(option.folder);
        }
    }
    // TODO: is compare argument required?
    // eslint-disable-next-line @typescript-eslint/require-array-sort-compare
    folders = folders.sort().filter(function (x, i, a) { return !i || x !== a[i - 1]; });
    return folders;
};
/**
 * Turn a hex color (like #212342) into r,g,b values
 *
 * @param {string} hex - hex colour
 * @returns {array} r,g,b
 */
var hexToRgb = function (hex) {
    // https://stackoverflow.com/questions/5623838/rgb-to-hex-and-hex-to-rgb
    // Expand shorthand form (e.g. "03F") to full form (e.g. "0033FF")
    var shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
    hex = hex.replace(shorthandRegex, function (m, r, g, b) {
        return r + r + g + g + b + b;
    });
    var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result
        ? {
            r: parseInt(result[1], 16),
            g: parseInt(result[2], 16),
            b: parseInt(result[3], 16)
        }
        : undefined;
};
/**
 * validate image and directory
 */
var validate = function (src, target) { return __awaiter(void 0, void 0, void 0, function () {
    var res;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0:
                if (!(target !== undefined)) return [3 /*break*/, 2];
                return [4 /*yield*/, fs_extra_1.ensureDir(target)];
            case 1:
                _a.sent();
                _a.label = 2;
            case 2: return [4 /*yield*/, checkSrc(src)];
            case 3:
                res = _a.sent();
                return [2 /*return*/, res];
        }
    });
}); };
// TODO: should take end param?
/**
 * Log progress in the command line
 *
 * @param {boolean} end
 */
var progress = function (msg) {
    process.stdout.write("  " + msg + "                       \r");
};
/**
 * Create a spinner on the command line
 *
 * @example
 *
 *     const spinnerInterval = spinner()
 *     // later
 *     clearInterval(spinnerInterval)
 */
var spinner = function () {
    if ('CI' in process.env || process.argv.some(function (arg) { return arg === '--ci'; })) {
        return null;
    }
    return setInterval(function () {
        process.stdout.write('/ \r');
        setTimeout(function () {
            process.stdout.write('- \r');
            setTimeout(function () {
                process.stdout.write('\\ \r');
                setTimeout(function () {
                    process.stdout.write('| \r');
                }, 100);
            }, 100);
        }, 100);
    }, 500);
};
var tauricon = (exports.tauricon = {
    validate: function (src, target) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, validate(src, target)];
                    case 1:
                        _a.sent();
                        return [2 /*return*/, typeof image === 'object'];
                }
            });
        });
    },
    version: function () {
        return package_json_1.version;
    },
    make: function (src, target, strategy, 
    // TODO: proper type for options
    options) {
        if (target === void 0) { target = path_1.default.resolve(app_paths_1.tauriDir, 'icons'); }
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!src) {
                            src = path_1.default.resolve(app_paths_1.appDir, 'app-icon.png');
                        }
                        spinnerInterval = spinner();
                        options = options || settings.options.tauri;
                        progress("Building Tauri icns and ico from \"" + src + "\"");
                        return [4 /*yield*/, this.validate(src, target)];
                    case 1:
                        _a.sent();
                        return [4 /*yield*/, this.icns(src, target, options, strategy)];
                    case 2:
                        _a.sent();
                        progress('Building Tauri png icons');
                        return [4 /*yield*/, this.build(src, target, options)];
                    case 3:
                        _a.sent();
                        if (!strategy) return [3 /*break*/, 5];
                        progress("Minifying assets with " + strategy);
                        return [4 /*yield*/, this.minify(target, options, strategy, 'batch')];
                    case 4:
                        _a.sent();
                        return [3 /*break*/, 6];
                    case 5:
                        log('no minify strategy');
                        _a.label = 6;
                    case 6:
                        progress('Tauricon Finished');
                        if (spinnerInterval)
                            clearInterval(spinnerInterval);
                        return [2 /*return*/, true];
                }
            });
        });
    },
    /**
     * Creates a set of images according to the subset of options it knows about.
     *
     * @param {string} src - image location
     * @param {string} target - where to drop the images
     * @param {object} options - js object that defines path and sizes
     */
    build: function (src, target, 
    // TODO: proper type for options
    options) {
        return __awaiter(this, void 0, void 0, function () {
            var sharpSrc, buildify2, output, folders, n, folder, _a, _b, _i, optionKey, option, _c, _d, _e, sizeKey, size, dest, pvar;
            return __generator(this, function (_f) {
                switch (_f.label) {
                    case 0: return [4 /*yield*/, this.validate(src, target)];
                    case 1:
                        _f.sent();
                        sharpSrc = sharp_1.default(src) // creates the image object
                        ;
                        buildify2 = function (pvar) {
                            return __awaiter(this, void 0, void 0, function () {
                                var pngImage, rgb, err_2;
                                return __generator(this, function (_a) {
                                    switch (_a.label) {
                                        case 0:
                                            _a.trys.push([0, 2, , 3]);
                                            pngImage = sharpSrc.resize(pvar[1], pvar[1]);
                                            if (pvar[2]) {
                                                rgb = hexToRgb(options.background_color) || {
                                                    r: undefined,
                                                    g: undefined,
                                                    b: undefined
                                                };
                                                pngImage.flatten({
                                                    background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 }
                                                });
                                            }
                                            pngImage.png();
                                            return [4 /*yield*/, pngImage.toFile(pvar[0])];
                                        case 1:
                                            _a.sent();
                                            return [3 /*break*/, 3];
                                        case 2:
                                            err_2 = _a.sent();
                                            warn(err_2);
                                            return [3 /*break*/, 3];
                                        case 3: return [2 /*return*/];
                                    }
                                });
                            });
                        };
                        folders = uniqueFolders(options);
                        // eslint-disable-next-line @typescript-eslint/no-for-in-array
                        for (n in folders) {
                            folder = folders[Number(n)];
                            // make the folders first
                            // TODO: should this be ensureDirSync?
                            // eslint-disable-next-line @typescript-eslint/no-floating-promises
                            fs_extra_1.ensureDir("" + target + path_1.default.sep + folder);
                        }
                        _a = [];
                        for (_b in options)
                            _a.push(_b);
                        _i = 0;
                        _f.label = 2;
                    case 2:
                        if (!(_i < _a.length)) return [3 /*break*/, 7];
                        optionKey = _a[_i];
                        option = options[String(optionKey)];
                        _c = [];
                        for (_d in option.sizes)
                            _c.push(_d);
                        _e = 0;
                        _f.label = 3;
                    case 3:
                        if (!(_e < _c.length)) return [3 /*break*/, 6];
                        sizeKey = _c[_e];
                        size = option.sizes[String(sizeKey)];
                        if (!!option.splash) return [3 /*break*/, 5];
                        dest = target + "/" + option.folder;
                        if (option.infix === true) {
                            output = "" + dest + path_1.default.sep + option.prefix + size + "x" + size + option.suffix;
                        }
                        else {
                            output = "" + dest + path_1.default.sep + option.prefix + option.suffix;
                        }
                        pvar = [
                            output,
                            size,
                            option.background
                        ];
                        return [4 /*yield*/, buildify2(pvar)];
                    case 4:
                        _f.sent();
                        _f.label = 5;
                    case 5:
                        _e++;
                        return [3 /*break*/, 3];
                    case 6:
                        _i++;
                        return [3 /*break*/, 2];
                    case 7: return [2 /*return*/];
                }
            });
        });
    },
    /**
     * Creates a set of splash images (COMING SOON!!!)
     *
     * @param {string} src - icon location
     * @param {string} splashSrc - splashscreen location
     * @param {string} target - where to drop the images
     * @param {object} options - js object that defines path and sizes
     */
    splash: function (src, splashSrc, target, 
    // TODO: proper type for options
    options) {
        return __awaiter(this, void 0, void 0, function () {
            var output, block, rgb, sharpSrc, data, _a, _b, _i, optionKey, option, _c, _d, _e, sizeKey, size, dest, pvar, sharpData;
            return __generator(this, function (_f) {
                switch (_f.label) {
                    case 0:
                        block = false;
                        rgb = hexToRgb(options.background_color) || {
                            r: undefined,
                            g: undefined,
                            b: undefined
                        };
                        if (splashSrc === src) {
                            // prevent overlay or pure
                            block = true;
                        }
                        if (!(block || options.splashscreen_type === 'generate')) return [3 /*break*/, 2];
                        return [4 /*yield*/, this.validate(src, target)];
                    case 1:
                        _f.sent();
                        if (!image) {
                            process.exit(1);
                        }
                        sharpSrc = sharp_1.default(src);
                        sharpSrc
                            .extend({
                            top: 726,
                            bottom: 726,
                            left: 726,
                            right: 726,
                            background: {
                                r: rgb.r,
                                g: rgb.g,
                                b: rgb.b,
                                alpha: 1
                            }
                        })
                            .flatten({ background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 } });
                        return [3 /*break*/, 3];
                    case 2:
                        if (options.splashscreen_type === 'overlay') {
                            sharpSrc = sharp_1.default(splashSrc)
                                .flatten({ background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 } })
                                .composite([
                                {
                                    input: src
                                    // blend: 'multiply' <= future work, maybe just a gag
                                }
                            ]);
                        }
                        else if (options.splashscreen_type === 'pure') {
                            sharpSrc = sharp_1.default(splashSrc).flatten({
                                background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 }
                            });
                        }
                        else {
                            throw new Error("unknown options.splashscreen_type: " + options.splashscreen_type);
                        }
                        _f.label = 3;
                    case 3: return [4 /*yield*/, sharpSrc.toBuffer()];
                    case 4:
                        data = _f.sent();
                        _a = [];
                        for (_b in options)
                            _a.push(_b);
                        _i = 0;
                        _f.label = 5;
                    case 5:
                        if (!(_i < _a.length)) return [3 /*break*/, 11];
                        optionKey = _a[_i];
                        option = options[String(optionKey)];
                        _c = [];
                        for (_d in option.sizes)
                            _c.push(_d);
                        _e = 0;
                        _f.label = 6;
                    case 6:
                        if (!(_e < _c.length)) return [3 /*break*/, 10];
                        sizeKey = _c[_e];
                        size = option.sizes[String(sizeKey)];
                        if (!option.splash) return [3 /*break*/, 9];
                        dest = "" + target + path_1.default.sep + option.folder;
                        return [4 /*yield*/, fs_extra_1.ensureDir(dest)];
                    case 7:
                        _f.sent();
                        if (option.infix === true) {
                            output = "" + dest + path_1.default.sep + option.prefix + size + "x" + size + option.suffix;
                        }
                        else {
                            output = "" + dest + path_1.default.sep + option.prefix + option.suffix;
                        }
                        pvar = [output, size];
                        sharpData = sharp_1.default(data);
                        sharpData = sharpData.resize(pvar[1][0], pvar[1][1]);
                        return [4 /*yield*/, sharpData.toFile(pvar[0])];
                    case 8:
                        _f.sent();
                        _f.label = 9;
                    case 9:
                        _e++;
                        return [3 /*break*/, 6];
                    case 10:
                        _i++;
                        return [3 /*break*/, 5];
                    case 11: return [2 /*return*/];
                }
            });
        });
    },
    /**
     * Minifies a set of images
     *
     * @param {string} target - image location
     * @param {object} options - where to drop the images
     * @param {string} strategy - which minify strategy to use
     * @param {string} mode - singlefile or batch
     */
    minify: function (target, 
    // TODO: proper type for options
    options, strategy, mode) {
        return __awaiter(this, void 0, void 0, function () {
            var cmd, minify, minifier, _a, folders, _b, _c, _i, n, folder;
            var _this = this;
            return __generator(this, function (_d) {
                switch (_d.label) {
                    case 0:
                        minify = settings.options.minify;
                        if (!minify.available.find(function (x) { return x === strategy; })) {
                            strategy = minify.type;
                        }
                        switch (strategy) {
                            case 'optipng':
                                cmd = imagemin_optipng_1.default(minify.optipngOptions);
                                break;
                            case 'zopfli':
                                cmd = imagemin_zopfli_1.default(minify.zopfliOptions);
                                break;
                            default:
                                throw new Error('unknown strategy' + strategy);
                        }
                        minifier = function (pvar, cmd) { return __awaiter(_this, void 0, void 0, function () {
                            return __generator(this, function (_a) {
                                switch (_a.label) {
                                    case 0: return [4 /*yield*/, imagemin_1.default([pvar[0]], {
                                            destination: pvar[1],
                                            plugins: [cmd]
                                        }).catch(function (err) {
                                            warn(err);
                                        })];
                                    case 1:
                                        _a.sent();
                                        return [2 /*return*/];
                                }
                            });
                        }); };
                        _a = mode;
                        switch (_a) {
                            case 'singlefile': return [3 /*break*/, 1];
                            case 'batch': return [3 /*break*/, 3];
                        }
                        return [3 /*break*/, 8];
                    case 1: return [4 /*yield*/, minifier([target, path_1.default.dirname(target)], cmd)];
                    case 2:
                        _d.sent();
                        return [3 /*break*/, 9];
                    case 3:
                        folders = uniqueFolders(options);
                        _b = [];
                        for (_c in folders)
                            _b.push(_c);
                        _i = 0;
                        _d.label = 4;
                    case 4:
                        if (!(_i < _b.length)) return [3 /*break*/, 7];
                        n = _b[_i];
                        folder = folders[Number(n)];
                        log('batch minify:' + String(folder));
                        return [4 /*yield*/, minifier([
                                "" + target + path_1.default.sep + folder + path_1.default.sep + "*.png",
                                "" + target + path_1.default.sep + folder
                            ], cmd)];
                    case 5:
                        _d.sent();
                        _d.label = 6;
                    case 6:
                        _i++;
                        return [3 /*break*/, 4];
                    case 7: return [3 /*break*/, 9];
                    case 8:
                        warn('[ERROR] Minify mode must be one of [ singlefile | batch]');
                        process.exit(1);
                        _d.label = 9;
                    case 9: return [2 /*return*/, 'minified'];
                }
            });
        });
    },
    /**
     * Creates special icns and ico filetypes
     *
     * @param {string} src - image location
     * @param {string} target - where to drop the images
     * @param {object} options
     * @param {string} strategy
     */
    icns: function (src, target, 
    // TODO: proper type for options
    options, strategy) {
        return __awaiter(this, void 0, void 0, function () {
            var sharpSrc, buf, out, out2, err_3;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        _a.trys.push([0, 3, , 4]);
                        if (!image) {
                            process.exit(1);
                        }
                        return [4 /*yield*/, this.validate(src, target)];
                    case 1:
                        _a.sent();
                        sharpSrc = sharp_1.default(src);
                        return [4 /*yield*/, sharpSrc.toBuffer()];
                    case 2:
                        buf = _a.sent();
                        out = png2icons.createICNS(buf, png2icons.BICUBIC, 0);
                        if (out === null) {
                            throw new Error('Failed to create icon.icns');
                        }
                        fs_extra_1.ensureFileSync(path_1.default.join(target, '/icon.icns'));
                        fs_extra_1.writeFileSync(path_1.default.join(target, '/icon.icns'), out);
                        out2 = png2icons.createICO(buf, png2icons.BICUBIC, 0, true);
                        if (out2 === null) {
                            throw new Error('Failed to create icon.ico');
                        }
                        fs_extra_1.ensureFileSync(path_1.default.join(target, '/icon.ico'));
                        fs_extra_1.writeFileSync(path_1.default.join(target, '/icon.ico'), out2);
                        return [3 /*break*/, 4];
                    case 3:
                        err_3 = _a.sent();
                        console.error(err_3);
                        throw err_3;
                    case 4: return [2 /*return*/];
                }
            });
        });
    }
});
/* eslint-enable @typescript-eslint/restrict-template-expressions */
if (true) {
    if ( true && module.exports) {
        exports = module.exports = tauricon;
    }
    exports.tauricon = tauricon;
}


/***/ }),

/***/ "./src/helpers/app-paths.ts":
/*!**********************************!*\
  !*** ./src/helpers/app-paths.ts ***!
  \**********************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {


// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.resolve = exports.tauriDir = exports.appDir = void 0;
var fs_1 = __webpack_require__(/*! fs */ "fs");
var path_1 = __webpack_require__(/*! path */ "path");
var logger_1 = __importDefault(__webpack_require__(/*! ./logger */ "./src/helpers/logger.ts"));
var chalk_1 = __importDefault(__webpack_require__(/*! chalk */ "chalk"));
var warn = logger_1.default('tauri', chalk_1.default.red);
function resolvePath(basePath, dir) {
    return dir && path_1.isAbsolute(dir) ? dir : path_1.resolve(basePath, dir);
}
var getAppDir = function () {
    var dir = process.cwd();
    var count = 0;
    // only go up three folders max
    while (dir.length > 0 && !dir.endsWith(path_1.sep) && count <= 2) {
        if (fs_1.existsSync(path_1.join(dir, 'src-tauri', 'tauri.conf.json'))) {
            return dir;
        }
        count++;
        dir = path_1.normalize(path_1.join(dir, '..'));
    }
    warn("Couldn't find recognize the current folder as a part of a Tauri project");
    process.exit(1);
};
var appDir = getAppDir();
exports.appDir = appDir;
var tauriDir = path_1.resolve(appDir, 'src-tauri');
exports.tauriDir = tauriDir;
var resolveDir = {
    app: function (dir) { return resolvePath(appDir, dir); },
    tauri: function (dir) { return resolvePath(tauriDir, dir); }
};
exports.resolve = resolveDir;


/***/ }),

/***/ "./src/helpers/logger.ts":
/*!*******************************!*\
  !*** ./src/helpers/logger.ts ***!
  \*******************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {


// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
var chalk_1 = __importDefault(__webpack_require__(/*! chalk */ "chalk"));
var ms_1 = __importDefault(__webpack_require__(/*! ms */ "ms"));
var prevTime;
exports.default = (function (banner, color) {
    if (color === void 0) { color = chalk_1.default.green; }
    return function (msg) {
        var curr = +new Date();
        var diff = curr - (prevTime || curr);
        prevTime = curr;
        if (msg) {
            console.log(
            // TODO: proper typings for color and banner
            // eslint-disable-next-line @typescript-eslint/restrict-template-expressions, @typescript-eslint/no-unsafe-call
            " " + color(String(banner)) + " " + msg + " " + chalk_1.default.green("+" + ms_1.default(diff)));
        }
        else {
            console.log();
        }
    };
});


/***/ }),

/***/ "./src/helpers/tauricon.config.ts":
/*!****************************************!*\
  !*** ./src/helpers/tauricon.config.ts ***!
  \****************************************/
/***/ ((__unused_webpack_module, exports) => {


// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.options = void 0;
exports.options = {
    // folder determines in which path to drop the generated file
    // prefix is the first part of the generated file's name
    // infix adds e.g. '44x44' based on the size in sizes to the generated file's name
    // suffix adds a file-ending to the generated file's name
    // sizes determines the pixel width and height to use
    background_color: '#000074',
    theme_color: '#02aa9b',
    sharp: 'kernel: sharp.kernel.lanczos3',
    minify: {
        batch: false,
        overwrite: true,
        available: ['optipng', 'zopfli'],
        type: 'optipng',
        optipngOptions: {
            optimizationLevel: 4,
            paletteReduction: true
        },
        zopfliOptions: {
            transparent: true,
            more: true
        }
    },
    splash_type: 'generate',
    tauri: {
        linux: {
            folder: '.',
            prefix: '',
            infix: true,
            suffix: '.png',
            sizes: [32, 128]
        },
        linux_2x: {
            folder: '.',
            prefix: '128x128@2x',
            infix: false,
            suffix: '.png',
            sizes: [256]
        },
        defaults: {
            folder: '.',
            prefix: 'icon',
            infix: false,
            suffix: '.png',
            sizes: [512]
        },
        appx_logo: {
            folder: '.',
            prefix: 'StoreLogo',
            infix: false,
            suffix: '.png',
            sizes: [50]
        },
        appx_square: {
            folder: '.',
            prefix: 'Square',
            infix: true,
            suffix: 'Logo.png',
            sizes: [30, 44, 71, 89, 107, 142, 150, 284, 310]
        }
        // todo: look at capacitor and cordova for insight into what icons
        // we need for those distribution targets
    }
};


/***/ }),

/***/ "./package.json":
/*!**********************!*\
  !*** ./package.json ***!
  \**********************/
/***/ ((module) => {

module.exports = JSON.parse('{"name":"@tauri-apps/cli","version":"1.0.0-beta.6","description":"Command line interface for building Tauri apps","bin":{"tauri":"./bin/tauri.js"},"files":["bin","dist","scripts"],"funding":{"type":"opencollective","url":"https://opencollective.com/tauri"},"scripts":{"build":"rimraf ./dist && webpack --progress","build-release":"rimraf ./dist && cross-env NODE_ENV=production webpack","test":"jest --runInBand --no-cache --testPathIgnorePatterns=\\"(build|dev)\\"","pretest":"yarn build","prepublishOnly":"yarn build-release","test:local":"jest --runInBand","lint":"eslint --ext ts \\"./src/**/*.ts\\"","lint-fix":"eslint --fix --ext ts \\"./src/**/*.ts\\"","lint:lockfile":"lockfile-lint --path yarn.lock --type yarn --validate-https --allowed-hosts npm yarn","format":"prettier --write --end-of-line=auto \\"./**/*.{cjs,js,jsx,ts,tsx,html,css,json}\\" --ignore-path .gitignore","format:check":"prettier --check --end-of-line=auto \\"./**/*.{cjs,js,jsx,ts,tsx,html,css,json}\\" --ignore-path .gitignore"},"repository":{"type":"git","url":"git+https://github.com/tauri-apps/tauri.git"},"contributors":["Tauri Team <team@tauri-apps.org> (https://tauri.studio)"],"license":"Apache-2.0 OR MIT","bugs":{"url":"https://github.com/tauri-apps/tauri/issues"},"homepage":"https://github.com/tauri-apps/tauri#readme","publishConfig":{"access":"public"},"engines":{"node":">= 12.13.0","npm":">= 6.6.0","yarn":">= 1.19.1"},"dependencies":{"@tauri-apps/toml":"2.2.4","chalk":"4.1.1","cross-env":"7.0.3","cross-spawn":"7.0.3","fs-extra":"10.0.0","got":"11.8.2","imagemin":"8.0.0","imagemin-optipng":"8.0.0","imagemin-zopfli":"7.0.0","inquirer":"8.1.1","is-png":"3.0.0","minimist":"1.2.5","ms":"2.1.3","png2icons":"2.0.1","read-chunk":"3.2.0","semver":"7.3.5","sharp":"0.28.3","update-notifier":"5.1.0"},"devDependencies":{"@babel/core":"7.14.6","@babel/preset-env":"7.14.7","@babel/preset-typescript":"7.14.5","@types/cross-spawn":"6.0.2","@types/fs-extra":"9.0.12","@types/imagemin":"7.0.1","@types/imagemin-optipng":"5.2.1","@types/inquirer":"7.3.3","@types/ms":"0.7.31","@types/semver":"7.3.7","@types/sharp":"0.28.4","@typescript-eslint/eslint-plugin":"4.28.3","@typescript-eslint/parser":"4.28.3","babel-jest":"27.0.6","eslint":"7.30.0","eslint-config-prettier":"8.3.0","eslint-config-standard-with-typescript":"20.0.0","eslint-plugin-import":"2.23.4","eslint-plugin-lodash-template":"0.19.0","eslint-plugin-node":"11.1.0","eslint-plugin-promise":"5.1.0","eslint-plugin-security":"1.4.0","is-running":"2.1.0","jest":"27.0.6","jest-transform-toml":"1.0.0","lockfile-lint":"4.6.2","prettier":"2.3.2","promise":"8.1.0","raw-loader":"4.0.2","rimraf":"3.0.2","toml-loader":"1.0.0","ts-loader":"9.2.3","typescript":"4.3.5","webpack":"5.44.0","webpack-cli":"4.7.2","webpack-node-externals":"3.0.0"},"resolutions":{"**/lodash":"4.17.21","**/hosted-git-info":"4.0.2","**/normalize-url":"6.1.0","**/trim-newlines":"4.0.2"}}');

/***/ }),

/***/ "chalk":
/*!************************!*\
  !*** external "chalk" ***!
  \************************/
/***/ ((module) => {

module.exports = require("chalk");;

/***/ }),

/***/ "fs":
/*!*********************!*\
  !*** external "fs" ***!
  \*********************/
/***/ ((module) => {

module.exports = require("fs");;

/***/ }),

/***/ "fs-extra":
/*!***************************!*\
  !*** external "fs-extra" ***!
  \***************************/
/***/ ((module) => {

module.exports = require("fs-extra");;

/***/ }),

/***/ "globby":
/*!*************************!*\
  !*** external "globby" ***!
  \*************************/
/***/ ((module) => {

module.exports = require("globby");;

/***/ }),

/***/ "graceful-fs":
/*!******************************!*\
  !*** external "graceful-fs" ***!
  \******************************/
/***/ ((module) => {

module.exports = require("graceful-fs");;

/***/ }),

/***/ "imagemin-optipng":
/*!***********************************!*\
  !*** external "imagemin-optipng" ***!
  \***********************************/
/***/ ((module) => {

module.exports = require("imagemin-optipng");;

/***/ }),

/***/ "imagemin-zopfli":
/*!**********************************!*\
  !*** external "imagemin-zopfli" ***!
  \**********************************/
/***/ ((module) => {

module.exports = require("imagemin-zopfli");;

/***/ }),

/***/ "junk":
/*!***********************!*\
  !*** external "junk" ***!
  \***********************/
/***/ ((module) => {

module.exports = require("junk");;

/***/ }),

/***/ "ms":
/*!*********************!*\
  !*** external "ms" ***!
  \*********************/
/***/ ((module) => {

module.exports = require("ms");;

/***/ }),

/***/ "path":
/*!***********************!*\
  !*** external "path" ***!
  \***********************/
/***/ ((module) => {

module.exports = require("path");;

/***/ }),

/***/ "png2icons":
/*!****************************!*\
  !*** external "png2icons" ***!
  \****************************/
/***/ ((module) => {

module.exports = require("png2icons");;

/***/ }),

/***/ "read-chunk":
/*!*****************************!*\
  !*** external "read-chunk" ***!
  \*****************************/
/***/ ((module) => {

module.exports = require("read-chunk");;

/***/ }),

/***/ "replace-ext":
/*!******************************!*\
  !*** external "replace-ext" ***!
  \******************************/
/***/ ((module) => {

module.exports = require("replace-ext");;

/***/ }),

/***/ "sharp":
/*!************************!*\
  !*** external "sharp" ***!
  \************************/
/***/ ((module) => {

module.exports = require("sharp");;

/***/ }),

/***/ "strtok3":
/*!**************************!*\
  !*** external "strtok3" ***!
  \**************************/
/***/ ((module) => {

module.exports = require("strtok3");;

/***/ }),

/***/ "strtok3/lib/core":
/*!***********************************!*\
  !*** external "strtok3/lib/core" ***!
  \***********************************/
/***/ ((module) => {

module.exports = require("strtok3/lib/core");;

/***/ }),

/***/ "token-types":
/*!******************************!*\
  !*** external "token-types" ***!
  \******************************/
/***/ ((module) => {

module.exports = require("token-types");;

/***/ }),

/***/ "util":
/*!***********************!*\
  !*** external "util" ***!
  \***********************/
/***/ ((module) => {

module.exports = require("util");;

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			// no module.id needed
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/************************************************************************/
/******/ 	/* webpack/runtime/define property getters */
/******/ 	(() => {
/******/ 		// define getter functions for harmony exports
/******/ 		__webpack_require__.d = (exports, definition) => {
/******/ 			for(var key in definition) {
/******/ 				if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
/******/ 					Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/hasOwnProperty shorthand */
/******/ 	(() => {
/******/ 		__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/make namespace object */
/******/ 	(() => {
/******/ 		// define __esModule on exports
/******/ 		__webpack_require__.r = (exports) => {
/******/ 			if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 				Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 			}
/******/ 			Object.defineProperty(exports, '__esModule', { value: true });
/******/ 		};
/******/ 	})();
/******/ 	
/************************************************************************/
/******/ 	
/******/ 	// startup
/******/ 	// Load entry module and return exports
/******/ 	// This entry module is referenced by other modules so it can't be inlined
/******/ 	var __webpack_exports__ = __webpack_require__("./src/api/tauricon.ts");
/******/ 	
/******/ 	return __webpack_exports__;
/******/ })()
;
});
//# sourceMappingURL=tauricon.js.map