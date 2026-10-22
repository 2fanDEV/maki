import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { MouseEvent, useState } from "react";
import { FileTextIcon, FolderIcon, UploadIcon } from "lucide-react";
import { useDropzone } from "react-dropzone";
import { Button } from "@/components/ui/button";
import { getDocuments } from "@/lib/api";
import { cn } from "@/lib/utils";

function formatFileSize(bytes: number) {
  const units = ["B", "KB", "MB", "GB"];
  const unit = bytes === 0 ? 0 : Math.min(Math.floor(Math.log10(bytes) / 3), 3);
  return `${Number((bytes / 1000 ** unit).toFixed(1))} ${units[unit]}`;
}

export function UploadDocumentDialog({
  open,
  onOpenChange,
  selectedFiles,
  onFileSelect,
}: {
  open: boolean;
  selectedFiles: File[];
  onFileSelect: (files: File[]) => void;
    onOpenChange: (open: boolean) => void
}) {
  const [folderName, setFolderName] = useState("");
  const documentsClient = getDocuments();
  const { getRootProps, getInputProps, isDragActive, open: openFileDialog } = useDropzone({
    noClick: true,
    noKeyboard: true,
    multiple: true,
    onDropAccepted: onFileSelect,
  });
  const totalSize = selectedFiles.reduce((total, file) => total + file.size, 0);
  const conditionalFolderInput = (
    <div className="space-y-2">
      <label htmlFor="upload-folder-name" className="text-xs font-medium text-muted-foreground">
        Folder name
      </label>
      <div className="relative">
        <FolderIcon aria-hidden="true" className="pointer-events-none absolute top-3 left-3 size-4 text-muted-foreground" />
        <Input
          id="upload-folder-name"
          className="h-10 bg-background/40 pl-9"
          type="text"
          placeholder="Folder name"
          value={folderName}
          onChange={(e) => setFolderName(e.target.value)}
        />
      </div>
    </div>
  );

  function handleFileUpload(event: MouseEvent<HTMLButtonElement>): void {
    selectedFiles.map((file) => {
      documentsClient.uploadDocument({
        file,
        bucket_name: folderName,
        metadata: undefined,
      });
    });
    onOpenChange(false);
    console.log(open);
  }

  function onChangeComplete(complete: boolean): void {
    if (open && complete === true) {
      onFileSelect([]);
    }
  }

  return (
    <div>
      <Dialog defaultOpen={false} open={open} onOpenChange={onOpenChange} onOpenChangeComplete={onChangeComplete}>
        <DialogTrigger onClick={() => onOpenChange(!open)} />
        <form>
          <DialogContent
            {...getRootProps({
              role: "dialog",
              className: cn(
                "max-h-[min(48rem,calc(100svh-2rem))] grid-rows-[auto_minmax(0,1fr)_auto] gap-0 overflow-hidden rounded-2xl border border-dashed border-foreground/20 p-0 shadow-2xl transition-[border-color,box-shadow] sm:max-w-[34rem]",
                isDragActive && "border-foreground/60 ring-4 ring-foreground/10",
              ),
            })}
          >
            <div className="px-6 pt-6 pb-5">
              <input {...getInputProps()} />
              <DialogTitle className="pr-6 text-lg font-semibold tracking-tight">Upload documents</DialogTitle>
              <DialogDescription className="mt-2 leading-relaxed">
                Drop files anywhere in this dialog to select them.
              </DialogDescription>
            </div>
            <div className="min-h-0 space-y-5 overflow-y-auto px-6 pb-6">
              <button
                type="button"
                onClick={openFileDialog}
                className={cn(
                  "group flex w-full cursor-pointer flex-col items-center rounded-xl bg-muted/40 px-4 py-7 text-center outline-none transition-colors hover:bg-muted/70 focus-visible:ring-2 focus-visible:ring-ring",
                  isDragActive && "bg-muted/80",
                )}
              >
                <span className="mb-4 flex size-12 items-center justify-center rounded-2xl border bg-popover text-muted-foreground shadow-xs transition-colors group-hover:text-foreground">
                  <UploadIcon aria-hidden="true" className="size-5" strokeWidth={1.5} />
                </span>
                <span className="font-medium">{isDragActive ? "Drop your files here" : "Drag & drop your documents"}</span>
                <span className="mt-1 text-xs text-muted-foreground">
                  or <span className="text-foreground underline decoration-foreground/30 underline-offset-4">browse files</span> from your computer
                </span>
              </button>
              {selectedFiles.length > 0 && (
                <>
                  <div>
                    <div className="mb-3 flex items-center gap-2">
                      <h3 className="text-xs font-medium text-muted-foreground">Selected files</h3>
                      <span className="rounded-md bg-muted px-1.5 py-0.5 text-[11px] font-medium tabular-nums">{selectedFiles.length}</span>
                    </div>
                    <ul className="divide-y overflow-hidden rounded-xl border bg-background/30">
                      {selectedFiles.map((file, index) => {
                        const extension = file.name.includes(".") ? file.name.split(".").pop()?.toUpperCase() : undefined;
                        return (
                          <li key={`${file.name}-${file.lastModified}-${index}`} className="flex items-center gap-3 px-3.5 py-3">
                            <span className="flex size-10 shrink-0 items-center justify-center rounded-lg border bg-popover text-muted-foreground">
                              <FileTextIcon aria-hidden="true" className="size-5" strokeWidth={1.5} />
                            </span>
                            <div className="min-w-0 flex-1">
                              <p className="truncate text-sm font-medium" title={file.name}>{file.name}</p>
                              <p className="mt-0.5 truncate text-xs text-muted-foreground">{extension ? `${extension} file` : file.type || "File"}</p>
                            </div>
                            <span className="shrink-0 text-xs text-muted-foreground tabular-nums">{formatFileSize(file.size)}</span>
                          </li>
                        );
                      })}
                    </ul>
                  </div>
                  {conditionalFolderInput}
                </>
              )}
            </div>
            <div className="flex items-center justify-between gap-4 border-t bg-muted/20 px-6 py-4">
              <div className="text-xs text-muted-foreground">
                {selectedFiles.length > 0 ? (
                  <>Total size <span className="ml-1 font-medium text-foreground tabular-nums">{formatFileSize(totalSize)}</span></>
                ) : "No files selected"}
              </div>
              <Button onClick={handleFileUpload} className="h-9 gap-2 rounded-lg px-4">
                <UploadIcon aria-hidden="true" className="size-4" />
                Upload
              </Button>
            </div>
          </DialogContent>
        </form>
      </Dialog>
    </div>
  );
}
