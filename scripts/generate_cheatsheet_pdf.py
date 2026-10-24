import os
from fpdf import FPDF

class SleekCheatsheetPDF(FPDF):
    def __init__(self):
        super().__init__(orientation='L', unit='mm', format='A4')
        self.set_auto_page_break(auto=False)
        self.add_font('JetBrainsMono', '', 'docs/assets/fonts/JetBrainsMono-Regular.ttf')
        self.add_font('JetBrainsMono', 'B', 'docs/assets/fonts/JetBrainsMono-Bold.ttf')
        
    def draw_card(self, x, y, w, h, title, accent_color, items, row_height):
        # Card container with smooth rounded corners
        self.set_fill_color(22, 27, 34)   # #161b22
        self.set_draw_color(48, 54, 61)   # #30363d
        self.set_line_width(0.25)
        self.rect(x, y, w, h, style='FD', round_corners=True, corner_radius=2.0)
        
        # Section header bar
        self.set_fill_color(27, 32, 40)
        self.rect(x, y, w, 6.8, style='F', round_corners=True, corner_radius=2.0)
        self.rect(x, y + 4.0, w, 2.8, style='F')
        
        # Accent indicator pill on left of title
        r, g, b = accent_color
        self.set_fill_color(r, g, b)
        self.rect(x + 3.0, y + 1.8, 2.0, 3.2, style='F', round_corners=True, corner_radius=0.5)
        
        # Section title
        self.set_font('JetBrainsMono', 'B', 7.2)
        self.set_text_color(240, 246, 252)  # #f0f6fc
        self.set_xy(x + 6.5, y + 1.1)
        self.cell(w - 10.0, 4.5, title, align='L')
        
        # Header divider line
        self.set_draw_color(33, 38, 45)
        self.set_line_width(0.2)
        self.line(x, y + 6.8, x + w, y + 6.8)
        
        # Uniform button dimensions and razor-straight text alignment
        badge_x = x + 2.8
        badge_w = 14.5
        badge_h = 3.6
        desc_x = badge_x + badge_w + 2.5
        desc_w = w - (desc_x - x) - 2.0
        
        cur_y = y + 8.2
        for key, desc in items:
            # Keycap button - UNIFORM DIMENSIONS
            self.set_fill_color(30, 36, 46)    # Dark slate button fill
            self.set_draw_color(56, 68, 88)    # Clean button border
            self.set_line_width(0.2)
            self.rect(badge_x, cur_y, badge_w, badge_h, style='FD', round_corners=True, corner_radius=0.8)
            
            # Key text in Electric Sky Blue, centered inside uniform button
            self.set_font('JetBrainsMono', 'B', 6.2)
            self.set_text_color(56, 189, 248) # #38bdf8 Electric Sky Blue
            self.set_xy(badge_x, cur_y + 0.1)
            self.cell(badge_w, 3.4, key, align='C')
            
            # Description text - PERFECT VERTICAL ALIGNMENT
            self.set_font('JetBrainsMono', '', 6.2)
            self.set_text_color(200, 210, 222) # High contrast off-white
            self.set_xy(desc_x, cur_y + 0.1)
            self.cell(desc_w, 3.4, desc, align='L')
            
            cur_y += row_height

def generate_pdf():
    os.makedirs('docs/assets', exist_ok=True)
    pdf = SleekCheatsheetPDF()
    pdf.add_page()
    
    # Background full page - Obsidian Dark
    pdf.set_fill_color(13, 17, 23) # #0d1117
    pdf.rect(0, 0, 297, 210, style='F')
    
    # Header area
    # Embed logo icon
    if os.path.exists('docs/assets/logo.png'):
        pdf.image('docs/assets/logo.png', x=10, y=5.2, w=7.8, h=7.8)
    
    # Title
    pdf.set_font('JetBrainsMono', 'B', 14)
    pdf.set_text_color(240, 246, 252) # #f0f6fc
    pdf.set_xy(19.5, 6.5)
    pdf.cell(32, 6, 'lazygitlab', align='L')
    
    # Tagline
    pdf.set_font('JetBrainsMono', '', 8.0)
    pdf.set_text_color(139, 148, 158) # #8b949e
    pdf.set_xy(53, 7.2)
    pdf.cell(100, 5, 'High-Performance Terminal UI for GitLab', align='L')
    
    # Info pill
    pdf.set_font('JetBrainsMono', 'B', 7.5)
    pdf.set_text_color(56, 189, 248) # #38bdf8
    pdf.set_xy(200, 7.5)
    pdf.cell(87, 5, 'OFFICIAL CHEATSHEET  •  v0.16.0', align='R')
    
    # Header divider
    pdf.set_draw_color(33, 38, 45) # #21262d
    pdf.set_line_width(0.3)
    pdf.line(10, 14.5, 287, 14.5)
    
    col_w = 66.2
    gap = 4.0
    x1 = 10.0
    x2 = x1 + col_w + gap
    x3 = x2 + col_w + gap
    x4 = x3 + col_w + gap
    
    y_top = 17.5
    h_top = 98.0
    y_bot = 119.5
    h_bot = 80.0
    
    # ================= COLUMN 1 =================
    pdf.draw_card(x1, y_top, col_w, h_top, 'GLOBAL & NAVIGATION', (56, 189, 248), [
        ('?', 'Command palette / Help'),
        ('Tab', 'Cycle panel focus forward'),
        ('S-Tab', 'Cycle panel focus backward'),
        ('j / k', 'Navigate cursor down / up'),
        ('h / l', 'Switch list / detail tab'),
        ('Ctrl+D', 'Half page scroll down'),
        ('Ctrl+U', 'Half page scroll up'),
        ('/', 'Filter items in real-time'),
        ('r', 'Force refresh view data'),
        ('p', 'Project quick switcher'),
        ('o', 'Open item in web browser'),
        ('y', 'Copy URL/text to clipboard'),
        ('f', 'Global fuzzy search'),
        ('I', 'Toggle involved-only items'),
        ('Esc', 'Close modal / Clear search'),
        ('q', 'Quit lazygitlab'),
    ], 5.4)
    
    pdf.draw_card(x1, y_bot, col_w, h_bot, 'QUICK JUMP MATRIX', (16, 185, 129), [
        ('g h', 'Jump to Home Launchpad'),
        ('g t', 'Jump to Personal To-Dos'),
        ('g m', 'Jump to Merge Requests'),
        ('g i', 'Jump to Project Issues'),
        ('g b', 'Jump to Kanban Boards'),
        ('g p', 'Jump to CI/CD Pipelines'),
        ('g r', 'Jump to Repository Tree'),
        ('g e', 'Jump to Explore Projects'),
        ('1..6', 'Jump to sidebar index 1-6'),
        ('d', 'Mark To-Do done (in To-Dos)'),
        ('s', 'Snooze To-Do for later'),
        ('w', 'Toggle project star/favorite'),
    ], 5.6)
    
    # ================= COLUMN 2 =================
    pdf.draw_card(x2, y_top, col_w, h_top, 'MERGE REQUESTS & ACTIONS', (56, 189, 248), [
        ('Enter', 'Open Diff Review view'),
        ('a / A', 'Approve Merge Request'),
        ('u / R', 'Unapprove Merge Request'),
        ('m / M', 'Merge confirmation modal'),
        ('E', 'Remote fast-forward rebase'),
        ('D / t', 'Toggle Draft (WIP) state'),
        ('S', 'Toggle Squash on merge'),
        ('Y', 'Copy source branch name'),
        ('P', 'Open Head Pipeline jobs'),
        ('b', 'Create branch from MR source'),
        ('x / c', 'Close / Reopen MR'),
        ('n', 'Create new Merge Request'),
        ('l', 'Assign / edit Labels modal'),
        ('m', 'Assign Milestone modal'),
        ('a', 'Assign Reviewers/Assignees'),
    ], 5.6)
    
    pdf.draw_card(x2, y_bot, col_w, h_bot, 'DIFF REVIEW & DISCUSSIONS', (16, 185, 129), [
        ('] / [', 'Jump next / prev diff file'),
        ('Space', 'Collapse / expand diff file'),
        ('c', 'Add inline comment on line'),
        ('v', 'Start visual range selection'),
        ('s / S', 'Resolve / unresolve thread'),
        ('r', 'Reply to discussion thread'),
        ('d', 'Toggle split / unified diff'),
        ('Ctrl+S', 'Submit comment in editor'),
        ('y', 'Copy diff hunk to clipboard'),
        ('Esc', 'Exit diff review / Discard'),
    ], 6.5)
    
    # ================= COLUMN 3 =================
    pdf.draw_card(x3, y_top, col_w, h_top, 'ISSUES & TRIAGE', (16, 185, 129), [
        ('n', 'Create new project Issue'),
        ('x', 'Close / Reopen issue'),
        ('l', 'Open Labels picker modal'),
        ('m', 'Assign Milestone modal'),
        ('a', 'Assign issue to user(s)'),
        ('V / M', 'Toggle group by Milestone'),
        ('e', 'Edit issue title/description'),
        ('w', 'Set issue weight score'),
        ('d', 'Set due date modal'),
        ('c', 'Add comment in discussions'),
        ('s', 'Resolve discussion note'),
        ('y', 'Copy issue web URL'),
        ('o', 'Open issue in browser'),
        ('r', 'Refresh issue metadata'),
        ('Esc', 'Return to issue list'),
    ], 5.6)
    
    pdf.draw_card(x3, y_bot, col_w, h_bot, 'SPRINT KANBAN BOARDS', (52, 211, 153), [
        ('h / l', 'Switch column (Left / Right)'),
        ('j / k', 'Navigate cards in column'),
        ('H', 'Move card to PREV list'),
        ('L', 'Move card to NEXT list'),
        ('Enter', 'Inspect issue details/notes'),
        ('n', 'Create issue inside column'),
        ('x', 'Close selected board issue'),
        ('l', 'Edit labels on board card'),
        ('m', 'Set milestone on board card'),
        ('a', 'Set assignee on board card'),
        ('r', 'Refresh board issues'),
        ('Esc', 'Return to main view'),
    ], 5.6)
    
    # ================= COLUMN 4 =================
    pdf.draw_card(x4, y_top, col_w, h_top, 'CI/CD PIPELINES & LOGS', (125, 211, 252), [
        ('Enter', 'Inspect stages / Open Log'),
        ('n', 'Trigger new pipeline run'),
        ('r / R', 'Retry failed job / pipeline'),
        ('c / C', 'Cancel running job/pipeline'),
        ('p / P', 'Play manual / scheduled job'),
        ('S', 'Pipeline Schedules popup'),
        ('v / V', 'Project CI/CD Variables'),
        ('y', 'Copy raw job log to clipboard'),
        ('S-j/k', 'Extend log text selection'),
        ('Tab', 'Toggle collapsible section'),
        ('/', 'Search log text (Enter=next)'),
        ('G/Home', 'Jump to bottom / top of log'),
        ('PgUp/Dn', 'Scroll log page by page'),
        ('Esc', 'Return to jobs list'),
    ], 6.0)
    
    pdf.draw_card(x4, y_bot, col_w, h_bot, 'REPOSITORY & BRANCHES', (16, 185, 129), [
        ('Enter', 'Open directory / view file'),
        ('b', 'Branch Switcher popup'),
        ('B', 'Create new git branch'),
        ('p', 'Project Quick Switcher'),
        ('j / k', 'Scroll file line by line'),
        ('Ctrl+D', 'Scroll half page down'),
        ('Ctrl+U', 'Scroll half page up'),
        ('y', 'Copy file path or content'),
        ('o', 'Open file in web browser'),
        ('Esc', 'Navigate up directory tree'),
    ], 6.5)
    
    # Footer info
    pdf.set_font('JetBrainsMono', '', 6.5)
    pdf.set_text_color(110, 118, 129) # #6e7681
    pdf.set_xy(10, 202.0)
    pdf.cell(277, 4, 'Tip: Press ? anytime inside lazygitlab to open the interactive Command Palette.  •  github.com/arnaudbletterer/lazygitlab', align='C')
    
    pdf.output('docs/assets/lazygitlab-cheatsheet.pdf')
    print('Generated sleek docs/assets/lazygitlab-cheatsheet.pdf successfully')

if __name__ == '__main__':
    generate_pdf()
