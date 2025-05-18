import { Component } from '@angular/core';
import {RouterLink, RouterLinkActive, RouterOutlet} from '@angular/router';
import {TranslateModule} from '@ngx-translate/core';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.scss'],
  imports: [RouterOutlet, TranslateModule, RouterLinkActive, RouterLink],
})
export class SettingsComponent {
  // This is a container component for the settings section
}
