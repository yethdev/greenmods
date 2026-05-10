import markdownit from "markdown-it";
import { full as emoji } from "markdown-it-emoji";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import csharp from "highlight.js/lib/languages/csharp";
import diff from "highlight.js/lib/languages/diff";
import ini from "highlight.js/lib/languages/ini";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import markdownLanguage from "highlight.js/lib/languages/markdown";
import plaintext from "highlight.js/lib/languages/plaintext";
import powershell from "highlight.js/lib/languages/powershell";
import rust from "highlight.js/lib/languages/rust";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";

hljs.registerLanguage("bash", bash);
hljs.registerAliases(["shell", "sh", "zsh"], { languageName: "bash" });

hljs.registerLanguage("csharp", csharp);
hljs.registerAliases(["cs", "dotnet"], { languageName: "csharp" });

hljs.registerLanguage("diff", diff);
hljs.registerAliases(["patch"], { languageName: "diff" });

hljs.registerLanguage("ini", ini);
hljs.registerLanguage("javascript", javascript);
hljs.registerAliases(["js", "jsx", "mjs", "cjs"], { languageName: "javascript" });

hljs.registerLanguage("json", json);
hljs.registerLanguage("markdown", markdownLanguage);
hljs.registerAliases(["md"], { languageName: "markdown" });

hljs.registerLanguage("plaintext", plaintext);
hljs.registerAliases(["text", "plain"], { languageName: "plaintext" });

hljs.registerLanguage("powershell", powershell);
hljs.registerAliases(["ps1", "pwsh"], { languageName: "powershell" });

hljs.registerLanguage("rust", rust);
hljs.registerAliases(["rs"], { languageName: "rust" });

hljs.registerAliases(["toml"], { languageName: "ini" });
hljs.registerLanguage("typescript", typescript);
hljs.registerAliases(["ts", "tsx"], { languageName: "typescript" });

hljs.registerLanguage("xml", xml);
hljs.registerAliases(["html", "svg"], { languageName: "xml" });

hljs.registerLanguage("yaml", yaml);
hljs.registerAliases(["yml"], { languageName: "yaml" });

const md = markdownit({
    html: false,
    xhtmlOut: false,
    breaks: false,
    langPrefix: "hljs-",
    linkify: true,
    typographer: true,
    quotes: "“”‘’",
    highlight: (str, lang) => {
        if (lang && hljs.getLanguage(lang)) {
            try {
                return hljs.highlight(str, { language: lang }).value;
            } catch (__) {}
        }

        return "";
    },
}).use(emoji);

export const markdownInline = (str: string): string => md.renderInline(str);
export const markdown = (str: string): string => md.render(str);
export const removeBase = (target: string, base: string) => target.replace(base, "");
