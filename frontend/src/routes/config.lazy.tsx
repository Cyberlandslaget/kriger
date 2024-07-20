import { createLazyFileRoute } from "@tanstack/react-router";
import MonacoEditor, { type Monaco } from "@monaco-editor/react";
import { useCallback, useEffect, useRef } from "react";
import { createHighlighter } from "shiki";
import { shikiToMonaco } from "@shikijs/monaco";
import theme from "shiki/themes/catppuccin-mocha.mjs";

export const Route = createLazyFileRoute("/config")({
  component: () => Configuration(),
});

function Configuration() {
  const stringConfiguration = useRef(`hello = [
  "world",
  "computer"
]`);

  const updateConfiguration = useCallback(() => {
    // TODO: post new configuration
  }, []);

  useEffect(() => {
    // TODO: fetch server configuration
  }, []);

  const monacoMount = useCallback(async (monaco: Monaco) => {
    const highlighter = await createHighlighter({
      themes: [
        {
          ...theme,
          name: "kriger",
          colors: {
            ...theme.colors,
            "editor.background": "#050a1b",
          },
        },
      ],
      langs: ["toml"],
    });
    monaco.languages.register({ id: "toml" });
    shikiToMonaco(highlighter, monaco);
  }, []);

  return (
    <main className="flex flex-col gap-3 h-full">
      <div className="h-96">
        <MonacoEditor
          value={stringConfiguration.current}
          beforeMount={monacoMount}
          language="toml"
          theme="kriger"
          onChange={(value?: string) => {
            if (!value) return;
            stringConfiguration.current = value;
          }}
          options={{
            minimap: {
              enabled: false,
            },
          }}
        />
      </div>

      <button
        type="button"
        className="flex items-center justify-center gap-2 bg-red-500/80 text-center truncate p-1 px-3 rounded-sm transition-all hover:!bg-red-600/60"
        onClick={updateConfiguration}
      >
        Update configuration
      </button>
    </main>
  );
}
export default Configuration;
