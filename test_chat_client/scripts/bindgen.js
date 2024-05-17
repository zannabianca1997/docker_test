'use strict'

const { execFileSync } = require('node:child_process');
const { join } = require('node:path');
const { exit } = require('node:process');
const { compile } = require('json-schema-to-typescript');
const { writeFileSync } = require('node:fs');

const FRONTEND_DIR = '../test_chat_server';
const BINDINGS_FILE = './app/bindings.d.ts';


function get_schema() {
    const manifest_dir = join(FRONTEND_DIR, 'Cargo.toml');
    try {
        return execFileSync('cargo', [
            'run',
            '--release',
            '--manifest-path', manifest_dir,
            '--features', 'bindgen',
            '--bin', 'bindgen'
        ], {
            stdio: ['ignore', 'pipe', 'inherit'],
            encoding: 'utf8',
        });
    } catch (err) {
        if (err.code) {
            console.error("Cannot launch cargo: " + err.code);
        } else {
            console.error("Error during bindgen, stopping.");
        }
        exit(1)
    }
}

// Get the api schema from the server project
const schema_str = get_schema();
// Parse it as a valid json
const schema = JSON.parse(schema_str);
// compile it
compile(schema, 'APIBindings').then(ts => {
    writeFileSync(BINDINGS_FILE, ts)
})


