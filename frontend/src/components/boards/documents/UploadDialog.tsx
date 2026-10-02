import { Dialog, DialogContent, DialogDescription, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";

export function UploadDocumentDialog({ open, onOpenChange, selectedFiles, onFileChange }: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  selectedFiles: File[] | null;
  onFileChange: (file: File | null) => void;
}) {
  return (
    <div>
      <Dialog defaultOpen={false} open={open} onOpenChange={onOpenChange}>
        <DialogTrigger onClick={() => onOpenChange(!open)}/>
        <form>
          <DialogContent>
            <DialogTitle>Document Upload</DialogTitle>
            <DialogDescription className="break-all">
               "Choose documents to upload."
            </DialogDescription>
            <Input
              multiple
              type="file"
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (file) onFileChange(file);
              }}
            />
          </DialogContent>
        </form>
      </Dialog>
    </div>
  );
}
