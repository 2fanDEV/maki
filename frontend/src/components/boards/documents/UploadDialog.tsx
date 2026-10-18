import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { DocumentDropzone } from "./DocumentDropzone";
import { MouseEvent, useState } from "react";
import { Button } from "@/components/ui/button";
import axios from 'axios';


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

  function handleFileUpload(event: MouseEvent<HTMLButtonElement>): void {

    onOpenChange(false);
    console.log(open);
  }

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
              <DocumentDropzone noClick={false}  onFileDrop={onFileSelect} dialog={true} />
            </div>
            <div className="flex justify-end">
              <Button onClick={handleFileUpload}  className="w-20">Upload</Button>
            </div>
          </DialogContent>
        </form>
      </Dialog>
    </div>
  );
}
