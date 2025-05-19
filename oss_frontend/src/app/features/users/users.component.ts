import {Component} from '@angular/core';
import {TranslatePipe} from '@ngx-translate/core';
import {RouterLink, RouterLinkActive, RouterOutlet} from '@angular/router';

@Component({
  selector: 'app-users',
  templateUrl: './users.component.html',
  imports: [TranslatePipe, RouterLink, RouterLinkActive, RouterOutlet],
  styleUrls: ['./users.component.scss'],
})
export class UsersComponent {
  // This is a placeholder component for the Users section
  // In a real application, this would include device management functionality
}
