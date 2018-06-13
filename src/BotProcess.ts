import {DaVinciBot} from "./DaVinciBot";
import {Idea}  from "./Idea";

export class BotProcess
{
    bot: DaVinciBot;
    rootIdea: Idea;
    status: BotStatus = BotStatus.Idle;
    commands: Array<BotCommand> = [];

    constructor(bot: DaVinciBot, rootIdea: Idea) {
        this.bot = bot;
        this.rootIdea = rootIdea;
        this.commands.push(new BotCommand('[Quit] this process.', [], (process: BotProcess) => { process.status = BotStatus.Idle; process.finish(); } ));
    }

    start(): void { }
    getOutput(): string { return '' }

    getCommandsList(): string {
        let output = '';

        for (let i = 1; i < this.commands.length; ++i) {
            // TODO also print out alternate keywords in pipe syntax
            output += `${i}. ${this.commands[i].description}\n`;
        }

        // TODO special H option to print command list
        // The last in the list is the special Quit command
        output += `Q. ${this.commands[0].description}\n`;

        return output;
    }

    handleInput(input: string): void { 
        let numInput = parseInt(input);
        if (isNaN(numInput)) {
            // TODO handle Q
            // TODO loop through checking against keywords to call it
        }
        else {
            // TODO call the command corresponding with numInput
            this.commands[numInput].event(this);
        }
        // TODO error checking
    }

    finish(): void { }
}

// A context-sensitive command
export class BotCommand
{
    // user-facing description of this command
    description: string;
    // alternate keywords with which to invoke this command
    keywords: Array<string>;
    // event triggered when the user calls this command
    event: (process: BotProcess) => void;

    // TODO scanning for keywords in brackets could be cleaner and give better
    // errors
    constructor(description: string, otherKeywords: Array<string> = [], event: (process: BotProcess)=>void=(process: BotProcess)=>null) {
        this.description = description;
        this.keywords = otherKeywords;
        this.event = event;

        // Any word in [brackets] in the description is automatically a keyword
        // TODO also allow [keyword|otherKeyword|...] syntax
        let start = 0;
        while (start < description.length) {
            let bracketIndex = description.indexOf('[', start);
            if (bracketIndex != -1) {
                let endBracketIndex = description.indexOf(']', bracketIndex);
                if (endBracketIndex != -1) {
                    let autoKeyword = description.substring(bracketIndex + 1, endBracketIndex);
                    this.keywords.push(autoKeyword);
                    /*console.log(autoKeyword);*/
                    start = endBracketIndex + 1;
                }
                else {
                    // Stop looking for keywords
                    start = description.length;
                }
            }
            else {
                break;
            }
        }
    }
}

export enum BotStatus
{
    HasOutput,
    NeedsInput,
    Idle
}
