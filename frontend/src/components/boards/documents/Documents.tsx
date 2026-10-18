import { Dropdown } from "@/components/Dropdown";
import { useState } from "react";
import { DocumentDropzone } from "./DocumentDropzone";
import { UploadDocumentDialog } from "./UploadDialog";

export function Documents() {
  const [uploadOpen, setUploadOpen] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);

  function handleFileDrop(files: File[]) {
    setUploadOpen(true);
    setSelectedFiles(files);
  }

  function handleUploadOpenChange(open: boolean) {
    setUploadOpen(open);
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
      <DocumentDropzone noClick={true} onFileDrop={handleFileDrop} dialog={false} />
      <UploadDocumentDialog
        open={uploadOpen}
        onOpenChange={handleUploadOpenChange}
        selectedFiles={selectedFiles}
        onFileSelect={setSelectedFiles}
      />
    </div>
  );
}
