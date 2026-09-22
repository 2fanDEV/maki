import { Dropdown } from "@/components/Dropdown";

const dropdownItems = [
  { id: "upload", label: "Upload", onSelect: () => { console.log("h1")}},
] as const;

export function Documents() {
  return (
    <div className="board-canvas">
      <div className="flex w-full items-center gap-4">
        <h1 id="board-title-documents" className="w-full">Documents</h1>
        <Dropdown
          items={dropdownItems}
          triggerLabel="Document actions"
        />
      </div>
    </div>
  );
}
