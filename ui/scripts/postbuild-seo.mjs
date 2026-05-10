import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const cwd = process.cwd();
const buildDir = path.join(cwd, "build");
const indexPath = path.join(buildDir, "index.html");
const projectsDir = path.join(buildDir, "p");
const searchDir = path.join(buildDir, "s");
const searchPath = path.join(searchDir, "index.html");
const sitemapPath = path.join(buildDir, "sitemap.xml");

const siteName = (process.env.PUBLIC_APP ?? "greenmods").toLowerCase();
const siteUrl = (process.env.PUBLIC_SITE_URL ?? "https://mods.yeth.dev").replace(/\/$/, "");
const seoApiBase = (process.env.GREENMODS_SEO_API_BASE ?? `${siteUrl}/api/v1`).replace(/\/$/, "");
const themeColor = process.env.PUBLIC_THEME_COLOR ?? "#16a34a";
const socialImage = `${siteUrl}/modhost.png`;
const platformDescription =
    "Open mod hosting for Subnautica, Below Zero, and Subnautica 2 with GitHub sync, collections, and compatibility notes.";

const home = {
    title: `${siteName} | GitHub-first mod hosting for Subnautica`,
    description: platformDescription,
    canonical: `${siteUrl}/`,
    robots: "index,follow,max-image-preview:large",
    ogTitle: `${siteName} | GitHub-first mod hosting for Subnautica`,
    ogDescription: platformDescription,
    subject: "Subnautica mod hosting",
    classification: "Game mod platform",
    audience: "Subnautica mod authors and players",
    jsonLd: {
        "@context": "https://schema.org",
        "@type": "WebSite",
        name: siteName,
        url: siteUrl,
        description: platformDescription,
        about: [
            "Subnautica mods",
            "GitHub releases",
            "mod collections",
            "compatibility notes",
        ],
        potentialAction: {
            "@type": "SearchAction",
            target: `${siteUrl}/s?q={search_term_string}`,
            "query-input": "required name=search_term_string",
        },
    },
};

const search = {
    title: `${siteName} Search | Subnautica 2 mod library`,
    description:
        "Search Subnautica 2 mods by loader, preview branch, tags, and release files in the main greenmods library.",
    canonical: `${siteUrl}/s`,
    robots: "noindex,follow,max-image-preview:large",
    ogTitle: `${siteName} Search | Subnautica 2 mod library`,
    ogDescription:
        "Search Subnautica 2 mods by loader, preview branch, tags, and release files in the main greenmods library.",
};

const normalizeArray = (value) => {
    if (Array.isArray(value)) {
        return value;
    }

    if (Array.isArray(value?.value)) {
        return value.value;
    }

    return [];
};

const cleanText = (value) =>
    (value ?? "")
        .replace(/\[(.*?)\]\((.*?)\)/g, "$1")
        .replace(/[_*`>#-]+/g, " ")
        .replace(/\s+/g, " ")
        .trim();

const trimText = (value, fallback, maxLength = 160) => {
    const cleaned = cleanText(value) || fallback;

    if (cleaned.length <= maxLength) {
        return cleaned;
    }

    const clipped = cleaned.slice(0, maxLength - 3);
    const lastSpace = clipped.lastIndexOf(" ");
    const safeClip = lastSpace > 80 ? clipped.slice(0, lastSpace) : clipped;

    return `${safeClip.trim()}...`;
};

const toAbsoluteUrl = (value, fallback = socialImage) => {
    if (!value) {
        return fallback;
    }

    try {
        return new URL(value, `${siteUrl}/`).toString();
    } catch {
        return fallback;
    }
};

const encodePathSegments = (...segments) => segments.map((segment) => encodeURIComponent(String(segment))).join("/");

const createMeta = ({
    title,
    description,
    canonical,
    ogTitle = title,
    ogDescription = description,
    ogType = "website",
    image = socialImage,
    robots = "index,follow,max-image-preview:large",
    keywords,
    jsonLd,
}) => ({
    title,
    description,
    canonical,
    robots,
    ogTitle,
    ogDescription,
    ogType,
    image,
    imageAlt: `Preview of ${ogTitle}`,
    keywords,
    jsonLd,
});

const escapeHtml = (value) =>
    String(value)
        .replaceAll("&", "&amp;")
        .replaceAll("<", "&lt;")
        .replaceAll(">", "&gt;")
        .replaceAll('"', "&quot;");

const buildSeoBlock = (meta) => {
    const tags = [
        `<title>${escapeHtml(meta.title)}</title>`,
        `<meta name="description" content="${escapeHtml(meta.description)}" />`,
        `<meta name="application-name" content="${escapeHtml(siteName)}" />`,
        meta.keywords ? `<meta name="keywords" content="${escapeHtml(meta.keywords)}" />` : null,
        meta.subject
            ? `<meta name="subject" content="${escapeHtml(meta.subject)}" />`
            : null,
        meta.classification
            ? `<meta name="classification" content="${escapeHtml(meta.classification)}" />`
            : null,
        meta.audience
            ? `<meta name="audience" content="${escapeHtml(meta.audience)}" />`
            : null,
        `<meta name="robots" content="${escapeHtml(meta.robots)}" />`,
        `<meta name="googlebot" content="${escapeHtml(meta.robots)}" />`,
        `<link rel="canonical" href="${escapeHtml(meta.canonical)}" />`,
        `<meta property="og:title" content="${escapeHtml(meta.ogTitle)}" />`,
        `<meta property="og:site_name" content="${escapeHtml(siteName)}" />`,
        `<meta property="og:type" content="${escapeHtml(meta.ogType ?? "website")}" />`,
        `<meta property="og:url" content="${escapeHtml(meta.canonical)}" />`,
        `<meta property="og:image" content="${escapeHtml(meta.image ?? socialImage)}" />`,
        `<meta property="og:image:alt" content="${escapeHtml(meta.imageAlt ?? "Preview of the greenmods mod library")}" />`,
        `<meta property="og:description" content="${escapeHtml(meta.ogDescription)}" />`,
        `<meta property="og:locale" content="en_US" />`,
        `<meta name="twitter:card" content="summary_large_image" />`,
        `<meta name="twitter:title" content="${escapeHtml(meta.ogTitle)}" />`,
        `<meta name="twitter:description" content="${escapeHtml(meta.ogDescription)}" />`,
        `<meta name="twitter:image" content="${escapeHtml(meta.image ?? socialImage)}" />`,
        `<meta name="twitter:image:alt" content="${escapeHtml(meta.imageAlt ?? "Preview of the greenmods mod library")}" />`,
        `<meta name="theme-color" content="${escapeHtml(themeColor)}" />`,
    ].filter(Boolean);

    if (meta.jsonLd) {
        tags.push(
            `<script type="application/ld+json">${JSON.stringify(meta.jsonLd)}</script>`,
        );
    }

    return tags.map((tag) => `        ${tag}`).join("\n");
};

const injectSeo = (html, meta) => {
    const seoBlock = buildSeoBlock(meta);

    return html.replace("</head>", `${seoBlock}\n    </head>`);
};

const injectNoScript = (html, noScript) => {
    if (!noScript) {
        return html;
    }

    return html.replace("</body>", `    <noscript>${noScript}</noscript>\n    </body>`);
};

const renderHtmlPage = (shell, meta, noScript) => injectNoScript(injectSeo(shell, meta), noScript);

const fetchJson = async (url) => {
    const response = await fetch(url, {
        headers: {
            Accept: "application/json",
        },
    });

    if (!response.ok) {
        throw new Error(`Failed to fetch ${url}: ${response.status}`);
    }

    return response.json();
};

const fetchProjects = async () => {
    const projects = [];
    let page = 1;
    let pages = 1;

    while (page <= pages) {
        const url = new URL(`${seoApiBase}/projects/search`);
        url.searchParams.set("sort", "updated");
        url.searchParams.set("dir", "desc");
        url.searchParams.set("page", String(page));
        url.searchParams.set("per_page", "100");
        url.searchParams.set("filters", "[]");

        const data = await fetchJson(url.toString());
        projects.push(...normalizeArray(data.results ?? data));
        pages = Number(data.pages ?? 1);
        page += 1;
    }

    return projects;
};

const buildProjectNoScript = (project, versions, projectTags, projectDescription) => {
    const authors = normalizeArray(project.authors).map((author) => author.username).join(", ");
    const versionLabels = versions.map((version) => version.version_number).filter(Boolean).join(", ");
    const tagLabels = projectTags.join(", ");
    const links = [project.source, project.issues, project.wiki]
        .filter(Boolean)
        .map((url) => `<li><a href="${escapeHtml(url)}">${escapeHtml(url)}</a></li>`)
        .join("");

    return `
<main>
  <h1>${escapeHtml(project.name)}</h1>
  <p>${escapeHtml(projectDescription)}</p>
  ${authors ? `<p>Authors: ${escapeHtml(authors)}</p>` : ""}
  ${tagLabels ? `<p>Tags: ${escapeHtml(tagLabels)}</p>` : ""}
  ${versionLabels ? `<p>Versions: ${escapeHtml(versionLabels)}</p>` : ""}
  ${links ? `<ul>${links}</ul>` : ""}
</main>`;
};

const buildVersionNoScript = (project, version, versionDescription) => {
    const fileNames = normalizeArray(version.files).map((file) => file.file_name).join(", ");

    return `
<main>
  <h1>${escapeHtml(project.name)} ${escapeHtml(version.version_number ?? version.name ?? "")}</h1>
  <p>${escapeHtml(versionDescription)}</p>
  ${fileNames ? `<p>Files: ${escapeHtml(fileNames)}</p>` : ""}
</main>`;
};

const buildSitemap = (entries) => {
    const lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
        ...entries.map(
            (entry) => `  <url>\n    <loc>${escapeHtml(entry.loc)}</loc>${entry.lastmod ? `\n    <lastmod>${escapeHtml(entry.lastmod)}</lastmod>` : ""}${entry.changefreq ? `\n    <changefreq>${escapeHtml(entry.changefreq)}</changefreq>` : ""}${entry.priority ? `\n    <priority>${escapeHtml(entry.priority)}</priority>` : ""}\n  </url>`,
        ),
        '</urlset>',
    ];

    return lines.join("\n");
};

const writeProjectPages = async (shell) => {
    let projects = [];

    try {
        projects = await fetchProjects();
    } catch (error) {
        console.warn(`SEO project generation skipped: ${error.message}`);
        return [];
    }

    const sitemapEntries = [
        {
            loc: `${siteUrl}/`,
            changefreq: "daily",
            priority: "1.0",
        },
    ];

    for (const summary of projects) {
        const slug = summary.slug;
        if (!slug) {
            continue;
        }

        let project;
        let versions = [];
        let gallery = [];

        try {
            [project, versions, gallery] = await Promise.all([
                fetchJson(`${seoApiBase}/projects/${encodeURIComponent(slug)}`),
                fetchJson(`${seoApiBase}/projects/${encodeURIComponent(slug)}/versions`).then(normalizeArray),
                fetchJson(`${seoApiBase}/projects/${encodeURIComponent(slug)}/gallery`).then(normalizeArray),
            ]);
        } catch (error) {
            console.warn(`Skipping SEO page for ${slug}: ${error.message}`);
            continue;
        }

        const projectPath = `/p/${encodePathSegments(slug)}`;
        const projectUrl = `${siteUrl}${projectPath}`;
        const projectImage = toAbsoluteUrl(gallery[0]?.url, socialImage);
        const projectTags = normalizeArray(project.tags).map((tag) => cleanText(tag)).filter(Boolean);
        const projectKeywords = [...new Set([project.name, ...projectTags, "compatibility", "search", "open source"])].join(", ");
        const projectDescription = trimText(
            project.description || project.readme,
            `Compatibility details, metadata, and release information for ${project.name}.`,
        );
        const projectMeta = createMeta({
            title: `${project.name} | ${siteName}`,
            description: projectDescription,
            canonical: projectUrl,
            ogType: "article",
            image: projectImage,
            keywords: projectKeywords,
            jsonLd: {
                "@context": "https://schema.org",
                "@type": "CreativeWork",
                name: project.name,
                description: projectDescription,
                url: projectUrl,
                image: projectImage,
                genre: "Software project",
                datePublished: project.created_at,
                dateModified: project.updated_at,
                keywords: projectTags,
                author: normalizeArray(project.authors).map((author) => ({
                    "@type": "Person",
                    name: author.username,
                })),
                sameAs: [project.source, project.issues, project.wiki].filter(Boolean),
            },
        });
        const projectHtml = renderHtmlPage(
            shell,
            projectMeta,
            buildProjectNoScript(project, versions, projectTags, projectDescription),
        );
        const projectFile = path.join(projectsDir, slug, "index.html");

        await mkdir(path.dirname(projectFile), { recursive: true });
        await writeFile(projectFile, projectHtml, "utf8");

        sitemapEntries.push({
            loc: projectUrl,
            lastmod: project.updated_at,
            changefreq: "weekly",
            priority: "0.8",
        });

        for (const version of versions) {
            if (!version?.id) {
                continue;
            }

            const versionPath = `/p/${encodePathSegments(slug, "versions", version.id)}`;
            const versionUrl = `${siteUrl}${versionPath}`;
            const versionDescription = trimText(
                version.changelog,
                `Release information for ${project.name} ${version.version_number ?? version.name ?? ""}.`,
            );
            const versionKeywords = [...new Set([project.name, version.version_number, ...(normalizeArray(version.loaders) ?? []), ...(normalizeArray(version.game_versions) ?? [])])]
                .filter(Boolean)
                .join(", ");
            const versionMeta = createMeta({
                title: `${project.name} ${version.version_number ?? version.name ?? version.id} | ${siteName}`,
                description: versionDescription,
                canonical: versionUrl,
                ogType: "article",
                image: projectImage,
                keywords: versionKeywords,
                jsonLd: {
                    "@context": "https://schema.org",
                    "@type": "CreativeWork",
                    name: `${project.name} ${version.version_number ?? version.name ?? version.id}`,
                    description: versionDescription,
                    url: versionUrl,
                    image: projectImage,
                    datePublished: version.created_at,
                    dateModified: version.updated_at,
                    isPartOf: projectUrl,
                },
            });
            const versionHtml = renderHtmlPage(
                shell,
                versionMeta,
                buildVersionNoScript(project, version, versionDescription),
            );
            const versionFile = path.join(projectsDir, slug, "versions", String(version.id), "index.html");

            await mkdir(path.dirname(versionFile), { recursive: true });
            await writeFile(versionFile, versionHtml, "utf8");

            sitemapEntries.push({
                loc: versionUrl,
                lastmod: version.updated_at ?? version.created_at,
                changefreq: "monthly",
                priority: "0.6",
            });
        }
    }

    return sitemapEntries;
};

const main = async () => {
    const shell = await readFile(indexPath, "utf8");
    const homeHtml = injectSeo(shell, home);
    const searchHtml = injectSeo(shell, search);
    const sitemapEntries = await writeProjectPages(shell);

    await writeFile(indexPath, homeHtml, "utf8");
    await mkdir(searchDir, { recursive: true });
    await writeFile(searchPath, searchHtml, "utf8");
    await writeFile(sitemapPath, buildSitemap(sitemapEntries), "utf8");
};

await main();