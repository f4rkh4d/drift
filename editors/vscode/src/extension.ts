import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

// Activates on first SQL file open or when a drift.toml lives in the workspace.
// Spawns `drift lsp` over stdio and routes SQL files through it. The drift
// binary handles dialect detection, rule running, fix code actions, and
// diagnostics. This extension is intentionally thin: the language server is
// the source of truth.
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const config = vscode.workspace.getConfiguration('drift');
  const driftPath = config.get<string>('path', 'drift');
  const dialect = config.get<string>('dialect', 'postgres');

  const driftAvailable = await isOnPath(driftPath);
  if (!driftAvailable) {
    vscode.window.showWarningMessage(
      `drift: '${driftPath}' was not found on your PATH. Install it with one of:\n` +
        `  brew install f4rkh4d/tap/drift\n` +
        `  cargo install drift-sql\n` +
        `  curl -fsSL https://drift.frkhd.com/install.sh | sh\n` +
        `Then reload the window.`,
    );
    return;
  }

  const serverOptions: ServerOptions = {
    run: {
      command: driftPath,
      args: ['lsp', '--dialect', dialect],
      transport: TransportKind.stdio,
    },
    debug: {
      command: driftPath,
      args: ['lsp', '--dialect', dialect],
      transport: TransportKind.stdio,
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'sql' }],
    synchronize: {
      // re-evaluate diagnostics if the user edits drift.toml or
      // .drift-baseline.json, since both change rule output.
      fileEvents: [
        vscode.workspace.createFileSystemWatcher('**/drift.toml'),
        vscode.workspace.createFileSystemWatcher('**/.drift-baseline.json'),
      ],
    },
    outputChannelName: 'drift',
  };

  client = new LanguageClient('drift', 'drift sql', serverOptions, clientOptions);
  await client.start();
  context.subscriptions.push(
    new vscode.Disposable(() => {
      void client?.stop();
    }),
  );
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}

async function isOnPath(bin: string): Promise<boolean> {
  // simple "does it execute" probe. avoids assuming a particular shell.
  return new Promise((resolve) => {
    const cp = require('child_process') as typeof import('child_process');
    const proc = cp.spawn(bin, ['--version'], { stdio: 'ignore' });
    proc.once('error', () => resolve(false));
    proc.once('exit', (code) => resolve(code === 0));
  });
}
