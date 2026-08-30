import os
from fpdf import FPDF

class SleekCheatsheetPDF(FPDF):
    def __init__(self):
        super().__init__(orientation='L', unit='mm', format='A4')
        self.set_auto_page_break(auto=False)
        self.add_font('JetBrainsMono', '', 'docs/assets/fonts/JetBrainsMono-Regular.ttf')
        self.add_font('JetBrainsMono', 'B', 'docs/assets/fonts/JetBrainsMono-Bold.ttf')
        
    def draw_card(self, x, y, w, h, title, accent_color, items):
        # Card container with smooth rounded corners
        self.set_fill_color(22, 27, 34)   # #161b22
        self.set_draw_color(48, 54, 61)   # #30363d
        self.set_line_width(0.25)
        self.rect(x, y, w, h, style='FD', round_corners=True, corner_radius=2.0)
        
        # Section header bar
        self.set_fill_color(27, 32, 40)
        self.rect(x, y, w, 6.5, style='F', round_corners=True, corner_radius=2.0)
        self.rect(x, y + 4.0, w, 2.5, style='F')
        
        # Accent indicator pill on left of title
        r, g, b = accent_color
        self.set_fill_color(r, g, b)
        self.rect(x + 3.0, y + 1.8, 2.0, 3.0, style='F', round_corners=True, corner_radius=0.5)
        
        # Section title
        self.set_font('JetBrainsMono', 'B', 7.5)
        self.set_text_color(240, 246, 252)  # #f0f6fc
        self.set_xy(x + 6.5, y + 1.0)
        self.cell(w - 10.0, 4.5, title, align='L')
        
        # Header divider line
        self.set_draw_color(33, 38, 45)
        self.set_line_width(0.2)
        self.line(x, y + 6.5, x + w, y + 6.5)
        
        # Rows
        cur_y = y + 7.8
        row_height = 4.9
        
        for key, desc in items:
            # Keycap badge
            key_text_w = self.get_string_width(key)
            badge_w = max(key_text_w + 3.0, 7.5)
            
            self.set_fill_color(33, 38, 45)    # #21262d
            self.set_draw_color(56, 62, 71)   # #383e47
            self.set_line_width(0.2)
            self.rect(x + 3.0, cur_y, badge_w, 3.6, style='FD', round_corners=True, corner_radius=0.8)
            
            # Key text
            self.set_font('JetBrainsMono', 'B', 6.6)
            self.set_text_color(240, 246, 252) # #f0f6fc
            self.set_xy(x + 3.0, cur_y + 0.1)
            self.cell(badge_w, 3.4, key, align='C')
            
            # Description text
            self.set_font('JetBrainsMono', '', 6.6)
            self.set_text_color(186, 194, 204) # #bac2cc
            self.set_xy(x + 4.2 + badge_w, cur_y + 0.1)
            self.cell(w - badge_w - 7.0, 3.4, desc, align='L')
            
            cur_y += row_height

def generate_pdf():
    os.makedirs('docs/assets', exist_ok=True)
    pdf = SleekCheatsheetPDF()
    pdf.add_page()
    
    # Background full page - Obsidian Dark
    pdf.set_fill_color(13, 17, 23) # #0d1117
    pdf.rect(0, 0, 297, 210, style='F')
    
    # Header area
    # Title
    pdf.set_font('JetBrainsMono', 'B', 15)
    pdf.set_text_color(252, 109, 38) # GitLab Fox Orange #fc6d26
    pdf.set_xy(10, 6.5)
    pdf.cell(38, 6, 'lazygitlab', align='L')
    
    # Tagline
    pdf.set_font('JetBrainsMono', '', 8.5)
    pdf.set_text_color(139, 148, 158) # #8b949e
    pdf.set_xy(48, 7.5)
    pdf.cell(100, 5, 'High-Performance Terminal UI for GitLab', align='L')
    
    # Info pill
    pdf.set_font('JetBrainsMono', 'B', 7.5)
    pdf.set_text_color(88, 166, 255) # #58a6ff
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
    
    # ================= COLUMN 1 =================
    pdf.draw_card(x1, 17.5, col_w, 98, 'GLOBAL & NAVIGATION', (88, 166, 255), [
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
    ])
    
    pdf.draw_card(x1, 118.5, col_w, 80, 'QUICK JUMP MATRIX', (188, 140, 255), [
        ('g t', 'Jump to Personal To-Dos'),
        ('g m', 'Jump to Merge Requests'),
        ('g i', 'Jump to Project Issues'),
        ('g b', 'Jump to Kanban Boards'),
        ('g p', 'Jump to CI/CD Pipelines'),
        ('g r', 'Jump to Repository Tree'),
        ('g e', 'Jump to Explore Projects'),
        ('g g', 'Jump to GitLab Groups'),
        ('1..6', 'Jump to menu index 1-6'),
        ('d', 'Mark To-Do done (in To-Dos)'),
        ('s', 'Snooze To-Do for later'),
        ('w', 'Toggle project star/favorite'),
    ])
    
    # ================= COLUMN 2 =================
    pdf.draw_card(x2, 17.5, col_w, 181, 'MERGE REQUESTS & CODE REVIEW', (247, 120, 186), [
        ('Enter', 'Open Diff Review view'),
        ('] / [', 'Jump next / prev diff file'),
        ('Space', 'Collapse / expand diff file'),
        ('c', 'Add inline comment on line'),
        ('v', 'Start visual range selection'),
        ('a / A', 'Approve Merge Request'),
        ('u / R', 'Unapprove Merge Request'),
        ('m / M', 'Merge confirmation modal'),
        ('E', 'Remote fast-forward rebase'),
        ('D / t', 'Toggle Draft (WIP) state'),
        ('S', 'Toggle Squash on merge'),
        ('Y', 'Copy source branch name'),
        ('P', 'Open Head Pipeline jobs'),
        ('x / c', 'Close / Reopen MR'),
        ('n', 'Create new Merge Request'),
        ('l', 'Assign / edit Labels modal'),
        ('m', 'Assign Milestone modal'),
        ('a', 'Assign Reviewers/Assignees'),
        ('b', 'Create branch from MR source'),
        ('s / S', 'Resolve / unresolve thread'),
        ('r', 'Reply to discussion thread'),
        ('Ctrl+S', 'Submit comment in editor'),
        ('Esc', 'Exit diff view / Discard'),
    ])
    
    # ================= COLUMN 3 =================
    pdf.draw_card(x3, 17.5, col_w, 98, 'ISSUES & TRIAGE', (250, 179, 135), [
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
    ])
    
    pdf.draw_card(x3, 118.5, col_w, 80, 'SPRINT KANBAN BOARDS', (63, 185, 80), [
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
    ])
    
    # ================= COLUMN 4 =================
    pdf.draw_card(x4, 17.5, col_w, 102, 'CI/CD PIPELINES & LIVE LOGS', (210, 153, 34), [
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
    ])
    
    pdf.draw_card(x4, 122.5, col_w, 76, 'REPOSITORY & BRANCHES', (57, 197, 187), [
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
    ])
    
    # Footer info
    pdf.set_font('JetBrainsMono', '', 6.5)
    pdf.set_text_color(110, 118, 129) # #6e7681
    pdf.set_xy(10, 201.5)
    pdf.cell(277, 4, 'Tip: Press ? anytime inside lazygitlab to open the interactive Command Palette.  •  github.com/abletterer/lazygitlab', align='C')
    
    pdf.output('docs/assets/lazygitlab-cheatsheet.pdf')
    print('Generated sleek docs/assets/lazygitlab-cheatsheet.pdf successfully')

if __name__ == '__main__':
    generate_pdf()
