const vscode = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function activate(context) {
    const serverCommand = '/home/alforce/Projeler/QW/build/app/test_qw';

    const serverOptions = {
        command: serverCommand,
        args: ["lsp"]
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'qw' }]
    };

    client = new LanguageClient(
        'QWLspServer',
        'QW Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = { activate, deactivate };
