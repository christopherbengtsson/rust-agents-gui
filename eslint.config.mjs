import babelParser from "@babel/eslint-parser";
import reactHooks from "eslint-plugin-react-hooks";
import jsxA11y from "eslint-plugin-jsx-a11y";

export default [
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      parser: babelParser,
      parserOptions: {
        requireConfigFile: false,
        babelOptions: {
          babelrc: false,
          configFile: false,
          parserOpts: { plugins: ["typescript", "jsx"] },
        },
      },
    },
    plugins: { "react-hooks": reactHooks, "jsx-a11y": jsxA11y },
    // Syntax linting complements tsc; Babel avoids typescript-eslint's current
    // unsupported TypeScript 7 peer range. Keep core TS checks in tsc.
    rules: {
      ...reactHooks.configs.recommended.rules,
      ...jsxA11y.flatConfigs.recommended.rules,
    },
  },
  {
    files: ["src/components/{ui,primitives}/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@tauri-apps/*", "@tauri-apps/**"],
              message:
                "Toolkit components receive data and callbacks; application adapters own Tauri access.",
            },
          ],
        },
      ],
      "no-restricted-syntax": [
        "error",
        {
          selector: "ImportExpression[source.value=/^@tauri-apps\\//]",
          message:
            "Toolkit components receive data and callbacks; application adapters own Tauri access.",
        },
        {
          selector:
            "ExportNamedDeclaration[source.value=/^@tauri-apps\\//], ExportAllDeclaration[source.value=/^@tauri-apps\\//]",
          message: "Keep Tauri re-exports outside toolkit components.",
        },
      ],
    },
  },
];
