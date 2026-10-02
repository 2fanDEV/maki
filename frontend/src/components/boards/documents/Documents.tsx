import { Dropdown } from "@/components/Dropdown";
import { useState } from "react";
import { DocumentDropzone } from "./DocumentDropzone";
import { UploadDocumentDialog } from "./UploadDialog";

export function Documents() {
  const [uploadOpen, setUploadOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  function handleFileDrop(file: File) {
    setSelectedFile(file);
    setUploadOpen(true);
  }

  function handleUploadOpenChange(open: boolean) {
    setUploadOpen(open);
    if (!open) setSelectedFile(null);
  }

  return (
    <div className="board-canvas h-full flex flex-col gap-4">
      <div className="flex w-full items-center gap-4">
        <h1 id="board-title-documents" className="w-full">Documents</h1>
        <Dropdown
          items={
            [
              { id: "upload", label: "Upload", onSelect: () => setUploadOpen(true)},
            ]
          }
          triggerLabel="Document actions"
        />

      </div>
      <DocumentDropzone disabled={uploadOpen} onFileDrop={handleFileDrop} />
      <UploadDocumentDialog
        open={uploadOpen}
        onOpenChange={handleUploadOpenChange}
        selectedFile={selectedFile}
        onFileChange={setSelectedFile}
      />
    </div>
  );
}
