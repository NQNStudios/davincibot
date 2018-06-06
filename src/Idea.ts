// The basic unit of things the user wants to do. Ideas are all created in
// unstructured form, and structured later by dividing and subdividing them
// into smaller Ideas.
export class Idea
{
    // Each Idea will have a unique id numbered upwards from 0.
    // TODO this will need to be serialized/deserialized when Tasks are reloaded from
    // a previous session!
    static Count: number = 0;

    id: number;
    name: string;
    description: string = '';
    tags: Array<string> = [];

    children: Array<Idea> = [];
    private _progress: number = 0;

    // Create an unstructured Idea
    constructor(name: string) {
        this.id = Idea.Count++;
        this.name = name;
    }

    // Check how near this Idea is to tangible completion
    get progress(): number {
        // Only Ideas without children can define their own progress
        if (this.children.length == 0) {
            return this._progress;
        }
        // If an Idea has children, its progress is measured as the average of
        // its children's progress values
        else {
            let sum = 0;
            for (let i = 0; i < this.children.length; ++i) {
                let child = this.children[i];
                sum += child.progress;
            }
            return sum / this.children.length;
        }
    }

    // Set how near this Idea is to tangible completion --
    // Only valid for an undivided Idea
    set progress(value: number) {
        if (this.children.length == 0) {
            if (value >= 0 && value <= 1) {
                this._progress = value;
            }
            else {
                throw new Error(`Tried to set an Idea's progress out of bounds [0:1]: ${value}`);
            }
        }
        else {
            throw new Error('Tried to directly set the progress of a task with children.');
        }
    }

    addChild(name: string) {
        let newChild = new Idea(name); 
        this.children.push(newChild);
        return newChild;
    }
}
