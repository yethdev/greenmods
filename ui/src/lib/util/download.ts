export const downloadFile = async (url: string, name?: string) => {
    const res = await fetch(url);

    if (!res.ok) {
        const message = await res.text().catch(() => "");

        throw new Error(message || `Download failed with status ${res.status}.`);
    }

    const data = await res.blob();
    const suggestedName = name ?? filenameFromDisposition(res.headers.get("content-disposition"));

    try {
        const handle = await window.showSaveFilePicker({
            suggestedName,
        });

        const stream = await handle.createWritable();

        await stream.write(data);
        await stream.close();
    } catch (err) {
        if ((err as Error).name == "AbortError") return;

        const a = document.createElement("a");
        const objectUrl = URL.createObjectURL(data);

        a.href = objectUrl;
        a.download = suggestedName ?? name ?? "";

        a.click();
        URL.revokeObjectURL(objectUrl);
    }
};

const filenameFromDisposition = (value: string | null) => {
    if (!value) return undefined;

    const match = value.match(/filename="([^"]+)"/i) ?? value.match(/filename=([^;]+)/i);

    return match?.[1]?.trim();
};
