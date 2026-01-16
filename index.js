#!/usr/bin/env node

import path from 'node:path';
import { existsSync } from 'node:fs';
import { styleText } from 'node:util';
import { _setPreopens } from '@bytecodealliance/preview2-shim/filesystem';
import { run } from './dist/openapi2mcp.js';

_setPreopens({'./': process.cwd() });

checkOutsidePaths();

run.run();


function checkOutsidePaths() {
    for (const arg of process.argv.slice(2)) {
        if (isOutsidePath(arg)) {
            const message = `Warning: The argument "${arg}" appears to be a path outside the current directory "${process.cwd()}". If this is intentional, note that only paths within the current directory are supported.`;
            console.warn(styleText('yellow', message));
        }
    }
}

function isOutsidePath(arg) {
    if (arg.startsWith('-')) {
        return false;
    }

    const looksLikePath = arg.includes('/') || arg.includes('\\') || arg.startsWith('.') || arg.startsWith('/') || existsSync(arg);

    if (!looksLikePath)
        return false;

    const resolvedPath = path.resolve(process.cwd(), arg);

    if (resolvedPath.startsWith(process.cwd()))
        return false;

    return true;
}
