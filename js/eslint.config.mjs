import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";

export default [
  {
    ignores: [
      "**/node_modules/**",
      "**/dist/**",
      "pkg/**",
      "pkg-*/**",
      "player/src/generated/**",
      "stat-evaluation-player/src/generated/**",
      "player/public/**",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{js,mjs}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    rules: {
      "no-useless-assignment": "off",
      "preserve-caught-error": "off",
      "no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    rules: {
      "no-undef": "off",
      "no-useless-assignment": "off",
      "preserve-caught-error": "off",
      "@typescript-eslint/no-empty-object-type": "off",
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/triple-slash-reference": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    // Seeded ballcam GameEngine modules, brought in largely verbatim. They carry
    // not-yet-wired subsystems (e.g. EffectsManager's goal/demo explosions) whose
    // helpers, classes, and interface-shaped params are intentionally retained
    // until those features are wired. Relax unused-vars here only; correctness
    // rules (no-undef, etc.) stay on. New first-party code lives in TS and is
    // linted strictly.
    files: ["player/src/viewer/managers/**/*.js", "player/src/viewer/lib/**/*.js"],
    rules: {
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": "off",
    },
  },
];
