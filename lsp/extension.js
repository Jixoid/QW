const vscode = require('vscode');

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

/**
 * Word Mapping Table (Kelime Eşleme Tablosu)
 * Maps language keywords, types, and constants directly to semantic token types.
 */
const WORD_MAP = new Map([
  // Declarations & Modifiers
  ['fun', 'keyword'],
  ['let', 'keyword'],
  ['var', 'keyword'],
  ['using', 'keyword'],
  ['struct', 'keyword'],
  ['iface', 'keyword'],
  ['trait', 'keyword'],
  ['enum', 'keyword'],
  ['flags', 'keyword'],
  ['impl', 'keyword'],
  ['generic', 'keyword'],
  ['mod', 'keyword'],
  ['pub', 'keyword'],
  ['priv', 'keyword'],
  ['prot', 'keyword'],
  ['crate', 'keyword'],
  ['use', 'keyword'],
  ['requires', 'keyword'],
  ['init', 'keyword'],
  ['new', 'keyword'],
  ['static', 'keyword'],
  ['const', 'keyword'],
  ['mut', 'keyword'],
  ['imm', 'keyword'],

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

  // Primitive & Standard Types
  ['i8', 'type'],
  ['i16', 'type'],
  ['i32', 'type'],
  ['i64', 'type'],
  ['u8', 'type'],
  ['u16', 'type'],
  ['u32', 'type'],
  ['u64', 'type'],
  ['f32', 'type'],
  ['f64', 'type'],
  ['bool', 'type'],
  ['str', 'type'],
  ['char', 'type'],
  ['void', 'type'],
  ['int', 'type'],
  ['uint', 'type'],
  ['float', 'type'],
  ['type', 'type'],
  ['Self', 'type'],

  // Special Variables & Literals
  ['self', 'variable'],
  ['true', 'number'],
  ['false', 'number'],
  ['null', 'number'],
  ['nil', 'number']
]);

/**
 * Highlighting provider operating strictly on Word Mapping lookup,
 * with support for function declarations (`fun X`), parameters (`(x, y: T)`),
 * type declarations (`struct X`), // line comments and /* */ (including nested) block comments.
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

    let inBlockComment = 0; // nested block comment depth across lines

    for (let lineIndex = 0; lineIndex < document.lineCount; lineIndex++) {
      const lineText = document.lineAt(lineIndex).text;
      let i = 0;
      const len = lineText.length;
      let expectFunctionName = false;
      let expectTypeName = false;

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
          } else if (tokenTypeStr) {
            const typeIndex = tokenTypes.indexOf(tokenTypeStr);
            if (typeIndex !== -1) {
              tokensBuilder.push(lineIndex, wordStart, word.length, typeIndex, 0);
            }
            if (word === 'fun' || word === 'init') {
              expectFunctionName = true;
            } else if (word === 'struct' || word === 'enum' || word === 'iface' || word === 'trait' || word === 'flags') {
              expectTypeName = true;
            }
          } else {
            // 1. Check if it's a function call (followed by '(')
            let peek = i;
            while (peek < len && /\s/.test(lineText[peek])) {
              peek++;
            }
            if (peek < len && lineText[peek] === '(' && functionTypeIdx !== -1) {
              tokensBuilder.push(lineIndex, wordStart, word.length, functionTypeIdx, 0);
            } else {
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
}

function deactivate() {}

module.exports = {
  activate,
  deactivate,
  WORD_MAP,
  tokenTypes,
  QWWordMappingHighlighter
};
