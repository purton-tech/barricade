export async function scrypt(password: ArrayBuffer, salt: ArrayBuffer, 
        N: number, r: number, p: number, dkLen: number, 
        progressCallback: (progress: number) => void) : Promise<ArrayBuffer> {
    return new Promise(function (resolve, reject) {
        let lastProgress = 0;
        if (progressCallback) { progressCallback(0); }
        _scrypt(password, salt, N, r, p, dkLen, function (error, progress, key) {
            if (error) {
                reject(error);
            } else if (key) {
                if (progressCallback && lastProgress !== 1) {
                    progressCallback(1);
                }
                resolve(new Uint8Array(key));
            } else if (progressCallback && progress !== lastProgress) {
                lastProgress = progress;
                return progressCallback(progress);
            }
        });
    });
}

const MAX_VALUE = 0x7fffffff;

// Returns a Promise that resolves to an ArrayBuffer.
async function PBKDF2_HMAC_SHA256(password, salt, iterations, dkLen): Promise<ArrayBuffer> {
    return crypto.subtle.importKey('raw', password, { name: 'PBKDF2' }, false, ["deriveBits"]).then(
        (pbkdf2) => crypto.subtle.deriveBits({
            name: 'PBKDF2',
            salt: salt,
            iterations: iterations,
            hash: { name: 'SHA-256' }
        }, pbkdf2, dkLen * 8)
    );
};

// The following is an adaptation of scryptsy
// See: https://www.npmjs.com/package/scryptsy
function blockmix_salsa8(BY, Yi, r, x, _X) {
    let i;

    arraycopy(BY, (2 * r - 1) * 16, _X, 0, 16);
    for (i = 0; i < 2 * r; i++) {
        blockxor(BY, i * 16, _X, 16);
        salsa20_8(_X, x);
        arraycopy(_X, 0, BY, Yi + (i * 16), 16);
    }

    for (i = 0; i < r; i++) {
        arraycopy(BY, Yi + (i * 2) * 16, BY, (i * 16), 16);
    }

    for (i = 0; i < r; i++) {
        arraycopy(BY, Yi + (i * 2 + 1) * 16, BY, (i + r) * 16, 16);
    }
}

function R(a, b) {
    return (a << b) | (a >>> (32 - b));
}

function salsa20_8(B, x) {
    arraycopy(B, 0, x, 0, 16);

    for (let i = 8; i > 0; i -= 2) {
        x[4] ^= R(x[0] + x[12], 7);
        x[8] ^= R(x[4] + x[0], 9);
        x[12] ^= R(x[8] + x[4], 13);
        x[0] ^= R(x[12] + x[8], 18);
        x[9] ^= R(x[5] + x[1], 7);
        x[13] ^= R(x[9] + x[5], 9);
        x[1] ^= R(x[13] + x[9], 13);
        x[5] ^= R(x[1] + x[13], 18);
        x[14] ^= R(x[10] + x[6], 7);
        x[2] ^= R(x[14] + x[10], 9);
        x[6] ^= R(x[2] + x[14], 13);
        x[10] ^= R(x[6] + x[2], 18);
        x[3] ^= R(x[15] + x[11], 7);
        x[7] ^= R(x[3] + x[15], 9);
        x[11] ^= R(x[7] + x[3], 13);
        x[15] ^= R(x[11] + x[7], 18);
        x[1] ^= R(x[0] + x[3], 7);
        x[2] ^= R(x[1] + x[0], 9);
        x[3] ^= R(x[2] + x[1], 13);
        x[0] ^= R(x[3] + x[2], 18);
        x[6] ^= R(x[5] + x[4], 7);
        x[7] ^= R(x[6] + x[5], 9);
        x[4] ^= R(x[7] + x[6], 13);
        x[5] ^= R(x[4] + x[7], 18);
        x[11] ^= R(x[10] + x[9], 7);
        x[8] ^= R(x[11] + x[10], 9);
        x[9] ^= R(x[8] + x[11], 13);
        x[10] ^= R(x[9] + x[8], 18);
        x[12] ^= R(x[15] + x[14], 7);
        x[13] ^= R(x[12] + x[15], 9);
        x[14] ^= R(x[13] + x[12], 13);
        x[15] ^= R(x[14] + x[13], 18);
    }

    for (let i = 0; i < 16; ++i) {
        B[i] += x[i];
    }
}

// naive approach... going back to loop unrolling may yield additional performance
function blockxor(S, Si, D, len) {
    for (let i = 0; i < len; i++) {
        D[i] ^= S[Si + i]
    }
}

function arraycopy(src, srcPos, dest, destPos, length) {
    // This doesn't seem to be any faster...
    //dest.set(src.subarray(srcPos, srcPos + length), destPos);
    while (length--) {
        dest[destPos++] = src[srcPos++];
    }
}

function checkBufferish(o) {
    return o instanceof Uint8Array || o instanceof ArrayBuffer;
}

function ensureInteger(value, name) {
    if (!Number.isInteger(value)) { throw new Error('invalid ' + name); }
    return value;
}

// N = Cpu cost, r = Memory cost, p = parallelization cost
// callback(error, progress, key)
async function _scrypt(password, salt, N, r, p, dkLen, callback) {

    N = ensureInteger(N, 'N');
    r = ensureInteger(r, 'r');
    p = ensureInteger(p, 'p');

    dkLen = ensureInteger(dkLen, 'dkLen');

    if (N === 0 || (N & (N - 1)) !== 0) { throw new Error('N must be power of 2'); }

    if (N > MAX_VALUE / 128 / r) { throw new Error('N too large'); }
    if (r > MAX_VALUE / 128 / p) { throw new Error('r too large'); }

    if (!checkBufferish(password)) {
        throw new Error('password must be an array or buffer');
    }

    if (!checkBufferish(salt)) {
        throw new Error('salt must be an array or buffer');
    }

    let b = await PBKDF2_HMAC_SHA256(password, salt, 1, p * 128 * r);
    // SubtleCrypto gives an ArrayBuffer, we need a Uint8Array.
    b = new Uint8Array(b);
    const B = new Uint32Array(p * 32 * r)
    for (let i = 0; i < B.length; i++) {
        const j = i * 4;
        B[i] = ((b[j + 3] & 0xff) << 24) |
            ((b[j + 2] & 0xff) << 16) |
            ((b[j + 1] & 0xff) << 8) |
            ((b[j + 0] & 0xff) << 0);
    }

    const XY = new Uint32Array(64 * r);
    const V = new Uint32Array(32 * r * N);

    const Yi = 32 * r;

    // scratch space
    const x = new Uint32Array(16);       // salsa20_8
    const _X = new Uint32Array(16);      // blockmix_salsa8

    const totalOps = p * N * 2;
    let currentOp = 0;
    let lastPercent10 = null;

    // Set this to true to abandon the scrypt on the next step
    let stop = false;

    // State information
    let state = 0;
    let i0 = 0, i1;
    let Bi;

    // How many blockmix_salsa8 can we do per step?
    const limit = callback ? parseInt(String(1000 / r)) : 0xffffffff;

    // This is really all I changed; making scryptsy a state machine so we occasionally
    // stop and give other evnts on the evnt loop a chance to run. ~RicMoo
    const incrementalSMix = async function () {
        if (stop) {
            return callback(new Error('cancelled'), currentOp / totalOps);
        }

        let steps;

        switch (state) {
            case 0:
                // for (var i = 0; i < p; i++)...
                Bi = i0 * 32 * r;

                arraycopy(B, Bi, XY, 0, Yi);                       // ROMix - 1

                state = 1;                                         // Move to ROMix 2
                i1 = 0;

            // Fall through

            case 1:

                // Run up to 1000 steps of the first inner smix loop
                steps = N - i1;
                if (steps > limit) { steps = limit; }
                for (let i = 0; i < steps; i++) {                  // ROMix - 2
                    arraycopy(XY, 0, V, (i1 + i) * Yi, Yi)         // ROMix - 3
                    blockmix_salsa8(XY, Yi, r, x, _X);             // ROMix - 4
                }

                // for (var i = 0; i < N; i++)
                i1 += steps;
                currentOp += steps;

                if (callback) {
                    // Call the callback with the progress (optionally stopping us)
                    const percent10 = parseInt(String(1000 * currentOp / totalOps));
                    if (percent10 !== lastPercent10) {
                        stop = callback(null, currentOp / totalOps);
                        if (stop) { break; }
                        lastPercent10 = percent10;
                    }
                }

                if (i1 < N) { break; }

                i1 = 0;                                          // Move to ROMix 6
                state = 2;

            // Fall through

            case 2:

                // Run up to 1000 steps of the second inner smix loop
                steps = N - i1;
                if (steps > limit) { steps = limit; }
                for (let i = 0; i < steps; i++) {                // ROMix - 6
                    const offset = (2 * r - 1) * 16;             // ROMix - 7
                    const j = XY[offset] & (N - 1);
                    blockxor(V, j * Yi, XY, Yi);                 // ROMix - 8 (inner)
                    blockmix_salsa8(XY, Yi, r, x, _X);           // ROMix - 9 (outer)
                }

                // for (var i = 0; i < N; i++)...
                i1 += steps;
                currentOp += steps;

                // Call the callback with the progress (optionally stopping us)
                if (callback) {
                    const percent10 = parseInt(String(1000 * currentOp / totalOps));
                    if (percent10 !== lastPercent10) {
                        stop = callback(null, currentOp / totalOps);
                        if (stop) { break; }
                        lastPercent10 = percent10;
                    }
                }

                if (i1 < N) { break; }

                arraycopy(XY, 0, B, Bi, Yi);                     // ROMix - 10

                // for (var i = 0; i < p; i++)...
                i0++;
                if (i0 < p) {
                    state = 0;
                    break;
                }

                b = new Uint8Array(B.byteLength);
                for (let i = 0, j = 0; i < B.length; i++) {
                    b[j++] = (B[i] >> 0);
                    b[j++] = (B[i] >> 8);
                    b[j++] = (B[i] >> 16);
                    b[j++] = (B[i] >> 24);
                }

                const derivedKey = PBKDF2_HMAC_SHA256(password, b, 1, dkLen);

                // Send the result to the callback
                if (callback) { callback(null, 1.0, await derivedKey); }

                // Done; don't break (which would reschedule)
                return derivedKey;
        }
    }

    do { // Run the smix state machine until completion
        var derivedKey = incrementalSMix();
    } while (typeof (await derivedKey) === 'undefined');

    return derivedKey;
}