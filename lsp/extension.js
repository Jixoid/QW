const vscode = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

/**
 * Token types legend for VSCode Semantic Highlighting
 */
const tokenTypes = [
  'keyword',   // 0
  'type',      // 1
  'class',     // 2
  'enum',      // 3
  'interface', // 4
  'variable',  // 5
  'parameter', // 6
  'property',  // 7
  'function',  // 8
  'number',    // 9
  'string',    // 10
  'comment',   // 11
  'operator'   // 12
];

const tokenModifiers = ['declaration', 'definition', 'readonly', 'static'];

const legend = new vscode.SemanticTokensLegend(tokenTypes, tokenModifiers);

const DECLARATION_KEYWORDS = new Set([
  'fun', 'task', 'init', 'fini', 'let', 'var', 'using', 'type',
  'struct', 'iface', 'trait', 'enum', 'flags', 'impl', 'generic',
  'mod', 'use', 'requires', 'static', 'const', 'mut', 'imm',
  'pub', 'priv', 'prot', 'crate', 'super',
  'self', 'Self'
]);

/**
 * Word Mapping Table (Kelime Eşleme Tablosu)
 * Maps language keywords, types, and constants directly to semantic token types.
 */
const WORD_MAP = new Map([
  // Declarations & Modifiers
  ['fun', 'keyword'],
  ['task', 'keyword'],
  ['init', 'keyword'],
  ['fini', 'keyword'],
  ['let', 'keyword'],
  ['var', 'keyword'],
  ['using', 'keyword'],
  ['type', 'keyword'],
  ['struct', 'keyword'],
  ['iface', 'keyword'],
  ['trait', 'keyword'],
  ['enum', 'keyword'],
  ['flags', 'keyword'],
  ['impl', 'keyword'],
  ['generic', 'keyword'],
  ['mod', 'keyword'],
  ['use', 'keyword'],
  ['requires', 'keyword'],
  ['static', 'keyword'],
  ['const', 'keyword'],
  ['mut', 'keyword'],
  ['imm', 'keyword'],

  // Visibility & Scoping
  ['pub', 'keyword'],
  ['priv', 'keyword'],
  ['prot', 'keyword'],
  ['crate', 'keyword'],
  ['super', 'keyword'],

  // Control Flow
  ['if', 'keyword'],
  ['ef', 'keyword'],
  ['else', 'keyword'],
  ['match', 'keyword'],
  ['loop', 'keyword'],
  ['while', 'keyword'],
  ['for', 'keyword'],
  ['in', 'keyword'],
  ['ret', 'keyword'],
  ['return', 'keyword'],
  ['break', 'keyword'],
  ['continue', 'keyword'],
  ['die', 'keyword'],

  // Primitive & Standard Types
  ['i8', 'type'],
  ['i16', 'type'],
  ['i32', 'type'],
  ['i64', 'type'],
  ['isize', 'type'],
  ['u8', 'type'],
  ['u16', 'type'],
  ['u32', 'type'],
  ['u64', 'type'],
  ['usize', 'type'],
  ['f32', 'type'],
  ['f64', 'type'],
  ['fsize', 'type'],
  ['bool', 'type'],
  ['str', 'type'],
  ['char', 'type'],
  ['void', 'type'],
  ['type', 'type'],

  // Special Variables & Literals
  ['self', 'keyword'],
  ['Self', 'keyword'],
  ['true', 'number'],
  ['false', 'number'],
  ['null', 'number'],
  ['nil', 'number']
]);

/**
 * Highlighting provider operating strictly on Word Mapping lookup,
 * with support for function declarations (`fun X`), parameters (`(x, y: T)`),
 * type declarations (`struct X`), line comments (//) and block comments.
 */
class QWWordMappingHighlighter {
  provideDocumentSemanticTokens(document) {
    const tokensBuilder = new vscode.SemanticTokensBuilder(legend);
    const commentTypeIdx = tokenTypes.indexOf('comment');
    const stringTypeIdx = tokenTypes.indexOf('string');
    const numberTypeIdx = tokenTypes.indexOf('number');
    const functionTypeIdx = tokenTypes.indexOf('function');
    const typeTypeIdx = tokenTypes.indexOf('type');
    const parameterTypeIdx = tokenTypes.indexOf('parameter');
    const variableTypeIdx = tokenTypes.indexOf('variable');

    let inBlockComment = 0; // nested block comment depth across lines
    let inImpl = false;

    for (let lineIndex = 0; lineIndex < document.lineCount; lineIndex++) {
      const lineText = document.lineAt(lineIndex).text;
      let i = 0;
      const len = lineText.length;
      let expectFunctionName = false;
      let expectTypeName = inImpl;
      let inLetVar = false;
      let expectVariableName = false;

      while (i < len) {
        // If we are currently inside a multi-line block comment
        if (inBlockComment > 0) {
          const commentStart = i;
          while (i < len && inBlockComment > 0) {
            if (lineText[i] === '/' && i + 1 < len && lineText[i + 1] === '*') {
              inBlockComment++;
              i += 2;
            } else if (lineText[i] === '*' && i + 1 < len && lineText[i + 1] === '/') {
              inBlockComment--;
              i += 2;
            } else {
              i++;
            }
          }
          if (commentTypeIdx !== -1 && i > commentStart) {
            tokensBuilder.push(lineIndex, commentStart, i - commentStart, commentTypeIdx, 0);
          }
          continue;
        }

        // Single-line comment //
        if (lineText[i] === '/' && i + 1 < len && lineText[i + 1] === '/') {
          if (commentTypeIdx !== -1) {
            tokensBuilder.push(lineIndex, i, len - i, commentTypeIdx, 0);
          }
          break; // Rest of the line is a comment
        }

        // Block comment start /*
        if (lineText[i] === '/' && i + 1 < len && lineText[i + 1] === '*') {
          const commentStart = i;
          inBlockComment = 1;
          i += 2;

          while (i < len && inBlockComment > 0) {
            if (lineText[i] === '/' && i + 1 < len && lineText[i + 1] === '*') {
              inBlockComment++;
              i += 2;
            } else if (lineText[i] === '*' && i + 1 < len && lineText[i + 1] === '/') {
              inBlockComment--;
              i += 2;
            } else {
              i++;
            }
          }

          if (commentTypeIdx !== -1 && i > commentStart) {
            tokensBuilder.push(lineIndex, commentStart, i - commentStart, commentTypeIdx, 0);
          }
          continue;
        }

        // String literals ("..." or '...')
        if (lineText[i] === '"' || lineText[i] === '\'') {
          const quote = lineText[i];
          const strStart = i;
          i++;
          while (i < len && lineText[i] !== quote) {
            if (lineText[i] === '\\' && i + 1 < len) {
              i += 2;
            } else {
              i++;
            }
          }
          if (i < len && lineText[i] === quote) {
            i++;
          }
          if (stringTypeIdx !== -1 && i > strStart) {
            tokensBuilder.push(lineIndex, strStart, i - strStart, stringTypeIdx, 0);
          }
          continue;
        }

        // Words (identifiers / keywords / types / functions / parameters)
        const char = lineText[i];
        if (/[a-zA-Z_]/.test(char)) {
          const wordStart = i;
          while (i < len && /[a-zA-Z0-9_]/.test(lineText[i])) {
            i++;
          }
          const word = lineText.slice(wordStart, i);
          const tokenTypeStr = WORD_MAP.get(word);

          if (expectFunctionName) {
            expectFunctionName = false;
            if (functionTypeIdx !== -1) {
              tokensBuilder.push(lineIndex, wordStart, word.length, functionTypeIdx, 1);
            }
          } else if (expectTypeName) {
            expectTypeName = false;
            if (typeTypeIdx !== -1) {
              tokensBuilder.push(lineIndex, wordStart, word.length, typeTypeIdx, 1);
            }
          } else if (expectVariableName) {
            expectVariableName = false;
            if (variableTypeIdx !== -1) {
              tokensBuilder.push(lineIndex, wordStart, word.length, variableTypeIdx, 1);
            }
          } else if (tokenTypeStr) {
            const typeIndex = tokenTypes.indexOf(tokenTypeStr);
            if (typeIndex !== -1) {
              const modMask = DECLARATION_KEYWORDS.has(word) ? 1 : 0;
              tokensBuilder.push(lineIndex, wordStart, word.length, typeIndex, modMask);
            }
            if (word === 'fun' || word === 'task' || word === 'init' || word === 'fini') {
              expectFunctionName = true;
            } else if (word === 'struct' || word === 'enum' || word === 'iface' || word === 'trait' || word === 'flags' || word === 'type') {
              expectTypeName = true;
            } else if (word === 'impl') {
              inImpl = true;
              expectTypeName = true;
            } else if (word === 'let' || word === 'var') {
              inLetVar = true;
              expectVariableName = true;
            }
          } else {
            // 1. Check if it's a function call (followed by '(')
            let peek = i;
            while (peek < len && /\s/.test(lineText[peek])) {
              peek++;
            }
            if (peek < len && lineText[peek] === '(' && functionTypeIdx !== -1) {
              tokensBuilder.push(lineIndex, wordStart, word.length, functionTypeIdx, 0);
            } else if (inLetVar) {
              if (variableTypeIdx !== -1) {
                tokensBuilder.push(lineIndex, wordStart, word.length, variableTypeIdx, 1);
              }
            } else if (!inImpl) {
              // 2. Check if it's a parameter before ':' (e.g. `v: T` or `x, y: T` or `min, max: &Self`)
              let isParam = false;
              let pIdx = i;
              while (pIdx < len) {
                while (pIdx < len && /\s/.test(lineText[pIdx])) pIdx++;
                if (pIdx < len && lineText[pIdx] === ':') {
                  isParam = true;
                  break;
                }
                if (pIdx < len && lineText[pIdx] === ',') {
                  pIdx++;
                  while (pIdx < len && /\s/.test(lineText[pIdx])) pIdx++;
                  if (pIdx < len && /[a-zA-Z_]/.test(lineText[pIdx])) {
                    while (pIdx < len && /[a-zA-Z0-9_]/.test(lineText[pIdx])) pIdx++;
                    continue;
                  }
                }
                break;
              }

              if (isParam && parameterTypeIdx !== -1) {
                tokensBuilder.push(lineIndex, wordStart, word.length, parameterTypeIdx, 1);
              }
            }
          }
          continue;
        }

        // Numbers
        if (/[0-9]/.test(char)) {
          const numStart = i;
          while (i < len && /[0-9.]/.test(lineText[i])) {
            i++;
          }
          if (numberTypeIdx !== -1 && i > numStart) {
            tokensBuilder.push(lineIndex, numStart, i - numStart, numberTypeIdx, 0);
          }
          continue;
        }

        // Other characters (symbols, operators, whitespace)
        if (char === '{' || char === ';') {
          inImpl = false;
          expectTypeName = false;
          inLetVar = false;
          expectVariableName = false;
        } else if (char === '=') {
          inLetVar = false;
          expectVariableName = false;
        } else if (inImpl && (char === ':' || char === ',' || char === '<')) {
          expectTypeName = true;
        } else if (inLetVar) {
          if (char === ':') {
            inLetVar = false;
            expectVariableName = false;
            expectTypeName = true;
          } else if (char === ',' || char === '(') {
            expectVariableName = true;
          }
        }
        i++;
      }
    }

    return tokensBuilder.build();
  }
}

/**
 * Extension activation entry point
 */
function activate(context) {
  const provider = new QWWordMappingHighlighter();
  const selector = { language: 'qw' };

  context.subscriptions.push(
    vscode.languages.registerDocumentSemanticTokensProvider(selector, provider, legend)
  );

  // LSP Client start
  client = createLanguageClient();
  client.start();

  // Register Restart Command
  context.subscriptions.push(
    vscode.commands.registerCommand('qw.restartServer', async () => {
      await restartLanguageServer();
    })
  );
}

function createLanguageClient() {
  const config = vscode.workspace.getConfiguration('qw');
  const serverCommand = config.get('lsp.serverPath') || 'qwd.debug';

  const serverOptions = {
    command: serverCommand,
    args: [],
    options: {
      shell: true
    }
  };

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'qw' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.qw')
    }
  };

  return new LanguageClient(
    'qw-lsp',
    'QW Language Server',
    serverOptions,
    clientOptions
  );
}

async function restartLanguageServer() {
  if (client) {
    try {
      await client.stop();
    } catch (err) {
      console.error('Failed to stop QW Language Server:', err);
    }
    client = undefined;
  }

  client = createLanguageClient();
  await client.start();
  vscode.window.showInformationMessage('QW Language Server restarted.');
}

function deactivate() {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

module.exports = {
  activate,
  deactivate,
  WORD_MAP,
  tokenTypes,
  QWWordMappingHighlighter
};
