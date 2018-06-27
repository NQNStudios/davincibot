import * as db from 'quick.db';

// The basic unit of user data in Da Vinci Bot.
// TODO write clear documentation on this
export class Idea = {
    readonly id: number;
    name: string = '';
    description: string = '';
    tags: Array<string> = [];
    children: Array<number> = [];

    private constructor(id: number) {
        this.id = id;
    }

    static async Get(id: number) {
        let idea = await db.fetch(id.toString());
    }
};

export class Ideas
{
    static async Initialize(): void {
        let ideaCount = await db.fetch('IdeaCount');
        if (ideaCount === null) {
            db.set('IdeaCount', 0);
            let rootIdea = NewIdea();
        }
    }

    static async NewIdea(): Idea {
        let ideaCount = await db.fetch('IdeaCount');
        db.add('IdeaCount', 1);

        let newIdea: Idea = { id: ideaCount }; 
        return newIdea;
    }
}

Ideas.Initialize();
