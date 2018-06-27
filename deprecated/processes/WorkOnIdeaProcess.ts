import {Idea} from "../Idea";
import {DaVinciBot} from "../DaVinciBot";
import {BotProcess, BotStatus, BotCommand} from "../BotProcess";
import {PlanIdeaProcess} from "./PlanIdeaProcess";

export class WorkOnIdeaProcess extends BotProcess
{
    start(): void {
        this.status = BotStatus.HasOutput;
    }

    constructor(bot: DaVinciBot, rootIdea: Idea) {
        super(bot, rootIdea);

        this.commands.push(new BotCommand('Mark this idea [done]', ['finish'], (process: BotProcess) => {
            process.rootIdea.progress = 1;
            process.status = BotStatus.Idle;
        }));

        this.commands.push(new BotCommand(`Work on this idea's [parts]`, [], (process) => {
            for (let i = process.rootIdea.children.length - 1; i >= 0; --i) {
                process.status = BotStatus.HasOutput;
                let child = process.rootIdea.children[i];
                if (child.progress != 1) {
                    process.bot.startProcess(new WorkOnIdeaProcess(process.bot, child));
                }
            }
        }));
    }

    getOutput(): string {
        this.status = BotStatus.NeedsInput;

        return `Working on idea: ${this.rootIdea.name}\n${this.getCommandsList()}`;

        // TODO add all of these options
        /*return `Working on idea ${this.rootIdea.name}:*/
            /*3. [Delete] this idea*/
            /*4. [Plan] this idea.*/
            /*5. [Clear] finished children*/
            /*6. [quit]*/
        /*`;*/
    }

    /*handleInput(input:string) {*/
        /*this.status = BotStatus.HasOutput;*/
        /*switch (input.toLowerCase()) {*/
            /*case '1':*/
            /*case 'done':*/
                /*break;*/
            /*case '2':*/
            /*case 'parts':*/
                /*break;*/
            /*case '3':*/
            /*case 'delete':*/
                /*// TODO delete this idea from its root*/
                /*break;*/
            /*case '4':*/
            /*case 'plan':*/
                /*this.bot.startProcess(new PlanIdeaProcess(this.bot, this.rootIdea));*/
            /*case '5':*/
            /*case 'clear':*/
                /*// TODO clear out all children with progress of 1*/
                /*break;*/
            /*case '6':*/
            /*case 'quit':*/
                /*this.status = BotStatus.Idle;*/
                /*break;*/
        /*}*/
    /*}*/

}
