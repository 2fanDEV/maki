import { useDropzone } from "react-dropzone";
import { cn } from "@/lib/utils";

type DocumentDropzoneProps = {
  disabled: boolean;
  onFileDrop: (file: File) => void;
};

export function DocumentDropzone({ disabled, onFileDrop }: DocumentDropzoneProps) {
  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    noClick: true,
    noKeyboard: true,
    multiple: false,
    disabled,
    onDropAccepted: ([file]) => {
      if (file) onFileDrop(file);
    },
  });

  return (
    <div
      {...getRootProps({
        className: cn(
          "flex-1 rounded-lg",
           isDragActive && "outline-2 outline-dashed outline-primary outline-offset-4",
        ),
      })}
    >
      <input {...getInputProps()} />
    </div>
  );
}
