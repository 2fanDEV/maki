import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { DocumentDropzone } from "./DocumentDropzone";
import { useState } from "react";

export function UploadDocumentDialog({
  open,
  onOpenChange,
  selectedFiles,
  onFileSelect,
}: {
  open: boolean;
  selectedFiles: File[];
  onFileSelect: (files: File[]) => void;
  onOpenChange: (open: boolean) => void;
}) {
  const [folderName, setFolderName] = useState("");
  const conditionalFolderInput = (
    <Input
      type="text"
      placeholder="Folder name"
      value={folderName}
      onChange={(e) => setFolderName(e.target.value)}
    />
  );

  return (
    <div>
      <Dialog defaultOpen={false} open={open} onOpenChange={onOpenChange}>
        <DialogTrigger onClick={() => onOpenChange(!open)} />
        <form>
          <DialogContent>
            <DialogTitle>Document Upload</DialogTitle>
            <DialogDescription className="break-all">
              Choose documents to upload
            </DialogDescription>
            {(selectedFiles.length ?? 0) > 0 && (conditionalFolderInput)}
            <div className="h-28">
              <DocumentDropzone onFileDrop={onFileSelect} dialog={true} />
            </div>
            <Input
              multiple
              type="file"
              onChange={(event) => {
                const file = event.target.files;
                if (file) {
                  const fileList = Array.from(file);
                  onFileSelect(fileList);
                  console.log(fileList);
                }
              }}
            />
          </DialogContent>
        </form>
      </Dialog>
    </div>
  );
}
