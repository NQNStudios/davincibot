// The basic unit of things the user wants to do. Ideas are all created in
// unstructured form, and structured later by dividing and subdividing them
// into smaller Ideas.
class Idea
{
    // Each Idea will have a unique id numbered upwards from 0.
    static Count: int = 0;

    id: int;
    name: string;
    description: string = '';
    tags: Array<string> = [];

    children: Array<Idea> = [];
    private _progress: float = 0;

    // Create an unstructured Idea
    constructor(name: string) {
        this.id = Count++;
        this.name = name;
    }

    // Check how near this Idea is to tangible completion
    get progress(): boolean {
        // Only Ideas without children can define their own progress
        if (this.children.length == 0) {
            return this._progress;
        }
        // If an Idea has children, its progress is measured as the average of
        // its children's progress values
        else {
            let sum = 0;
            for (var child: Idea in this.children) {
                sum += child.progress;
            }
            return sum / this.children.length;
        }
    }

    // Set how near this Idea is to tangible completion --
    // Only valid for an undivided Idea
    set progress(value): boolean {
        this._progress = value;
    }
}
