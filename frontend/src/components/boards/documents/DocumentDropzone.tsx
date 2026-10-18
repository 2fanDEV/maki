import { useDropzone } from "react-dropzone";
import { cn } from "@/lib/utils";

type DocumentDropzoneProps = {
  className?: string;
  dialog: boolean;
  noClick: boolean;
  disabled?: boolean;
  onFileDrop: (files: File[]) => void;
};

export function DocumentDropzone({ className, noClick, dialog, disabled = false, onFileDrop }: DocumentDropzoneProps) {
  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    noClick,
    noKeyboard: true,
    multiple: true,
    disabled,
    onDropAccepted: onFileDrop,
  });

  return (
    <div
      {...getRootProps({
        className: cn(
          className,
          "flex-1 rounded-2xl",
          (isDragActive || dialog) && "h-full outline-2 outline-dashed outline-grey outline-offset-4",
        ),
      })}
    >
      <input {...getInputProps()} />
    </div>
  );
}
